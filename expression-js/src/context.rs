use wasm_bindgen::{
    JsValue,
    UnwrapThrowExt,
    prelude::*
};
use crate::DataSource;

#[wasm_bindgen(js_name=Context)]
pub struct Context {
    cx: expression::Context<DataSource>,
}

/// This function attempts to convert a JS value into its native equivalent.
///
/// * `js_sys::String`s => `expression::Object::String`
/// * `js_sys::Array`s => `expression::Object::List`
/// * `js_sys::Object`s => `expression::Object::AssociativeArray`
/// * `js_sys::Object + { [Symbol.address]: Address }`s => `expression::Object::Address` !
/// * `js_sys::Null` => `expression::Object::Nothing`
/// * `js_sys::falsy()` => `expression::Object::Boolean(false)`
/// * `js_sys::truthy()` => `expression::Object::Boolean(true)`
///
/// > **Note:** Nested values such as those in lists or associative arrays which
/// fail automatic conversation will be dropped silently.
pub(crate) fn js_value_to_object(value: JsValue) -> Option<expression::Object> {
    Some(match value {
        value if value.is_string() => expression::Object::String(value.as_string()?),
        value if value.is_array() => expression::Object::List(js_sys::Array::from(&value)
            .into_iter()
            .flat_map(js_value_to_object)
            .collect()),

        // TODO: Detect Addresses
        value if value.is_object() => expression::Object::AssociativeArray(js_sys::Reflect::own_keys(&value).ok()?
            .into_iter()
            .flat_map(|i| match i {
                value if value.is_string() || value.is_symbol() => value.as_string(),
                _ => None
            })
            .flat_map(|key| js_value_to_object(js_sys::Reflect::get(&value, &JsValue::from_str(&key)).ok()?)
                .map(|value| (key.clone(), value)))
            .collect()),
        value if value.is_null() => expression::Object::Nothing,
        value if value.is_falsy() => expression::Object::Boolean(false),
        value if value.is_truthy() => expression::Object::Boolean(true),
        _ => None?,
    })
}


/// This function attempts to convert a native value into its JS equivalent.
///
/// * `js_sys::String`s => `expression::Object::String`
/// * `js_sys::Array`s => `expression::Object::List`
/// * `js_sys::Object`s => `expression::Object::AssociativeArray`
/// * `js_sys::Object + { [Symbol.address]: Address }`s => `expression::Object::Address` !
/// * `js_sys::Null` => `expression::Object::Nothing`
/// * `js_sys::falsy()` => `expression::Object::Boolean(false)`
/// * `js_sys::truthy()` => `expression::Object::Boolean(true)`
///
/// > **Note:** Nested values such as those in lists or associative arrays which
/// fail automatic conversation will be dropped silently.
pub(crate) fn value_to_js_object(value: expression::Object) -> Option<JsValue> {
    Some(match value {
        expression::Object::Number(num) => num.into(),
        expression::Object::Boolean(bool) => bool.into(),
        expression::Object::String(str) => JsValue::from_str(&str),
        expression::Object::List(list) => JsValue::from(js_sys::Array::from_iter(list.into_iter()
            .flat_map(value_to_js_object))),
        expression::Object::AssociativeArray(arr) => {
            let key_map = js_sys::Object::new();

            for (key, value) in arr {
                js_sys::Reflect::set(&key_map, &JsValue::from_str(&key), &value_to_js_object(value)?)
                    .unwrap_throw();
            }

            JsValue::from(key_map)
        },
        _ => todo!() // TODO: Addresses
    })
}

#[wasm_bindgen(js_class=Context)]
impl crate::Context {
    #[wasm_bindgen(constructor)]
    pub fn new(config: DataSource) -> crate::Context {
        Self {
            cx: expression::Context::new(config)
        }
    }

    #[wasm_bindgen(js_name = pushGlobal)]
    pub fn set_global(&mut self, name: js_sys::JsString, global: JsValue) {
        // Convert the global into the `expression::Object` equivalent

        self.cx.push_global(name.as_string().unwrap_throw(), js_value_to_object(global).unwrap_throw());
    }

    #[deprecated(note = "Use push_global instead")]
    #[wasm_bindgen(js_name = withGlobal)]
    pub fn with_global(self, name: js_sys::JsString, global: JsValue) -> Self {
        unimplemented!("Use the `pushGlobal` function instead`")
    }

    #[wasm_bindgen(js_name = pushOperator)]
    pub fn set_operator(&mut self, operator: Operator) {
        self.cx.push_operator();
    }

    #[deprecated(note = "Use push_operator instead")]
    #[wasm_bindgen(js_name = withOperator)]
    pub fn with_operator(self, name: js_sys::JsString, global: JsValue) -> Self {
        unimplemented!("Use the `pushOperator` function instead");
    }

    #[wasm_bindgen(js_name = evaluate)]
    pub fn evaluate(&self, expression: js_sys::JsString) -> JsValue {
        value_to_js_object(self.cx.evaluate(expression.as_string().unwrap_throw()).unwrap_throw()).unwrap_throw()
    }
}