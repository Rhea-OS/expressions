use wasm_bindgen::{JsValue, UnwrapThrowExt};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use expression::{Error, ManualError, Object};

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
pub(crate) fn js_value_to_object(value: JsValue) -> Option<Object> {
    if let Some(number) = value.as_f64() {
        return Some(Object::Number(number));
    }

    Some(match value {
        value if value.is_null() || value.is_undefined() => Object::Nothing,

        value if value.is_string() => Object::String(value.as_string()?),
        value if value.is_array() => Object::List(js_sys::Array::from(&value)
            .into_iter()
            .flat_map(js_value_to_object)
            .collect()),

        value if value.is_function() => {
            let value = value.clone();
            Object::function(move |args| {
                let args = js_sys::Array::from_iter(args.into_iter()
                    .map(value_to_js_object)
                    .map(|i| i.ok_or(Into::<Error>::into(ManualError::ConversionFailed)))
                    .collect::<expression::Result<Vec<_>>>()?
                    .into_iter());
                let result = js_sys::Function::from(value.clone()).call1(&JsValue::null(), &JsValue::from(args))
                    .unwrap_throw();
                js_value_to_object(result)
                    .ok_or(ManualError::ConversionFailed.into())
            })
        },

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

        value if value.is_falsy() => Object::Boolean(false),
        value if value.is_truthy() => Object::Boolean(true),
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
pub(crate) fn value_to_js_object(value: Object) -> Option<JsValue> {
    Some(match value {
        Object::Number(num) => js_sys::Number::from(num).into(),
        Object::Boolean(bool) => js_sys::Boolean::from(bool).into(),
        Object::String(str) => JsValue::from_str(&str),
        Object::List(list) => JsValue::from(js_sys::Array::from_iter(list.into_iter()
            .flat_map(value_to_js_object))),
        Object::AssociativeArray(arr) => {
            let key_map = js_sys::Object::new();

            for (key, value) in arr {
                js_sys::Reflect::set(&key_map, &JsValue::from_str(&key), &value_to_js_object(value)?)
                    .unwrap_throw();
            }

            JsValue::from(key_map)
        },
        Object::Nothing => JsValue::null(),
        Object::Function(function) => JsClosure::new(move |_args| -> JsValue {
            wasm_bindgen::throw_str("Fuck you");
            function(vec![])
                .map(value_to_js_object)
                .unwrap_throw()
                .unwrap_throw()
        }).into(),
    })
}


#[wasm_bindgen]
struct JsClosure {
    closure: Closure<dyn Fn()>,
}

impl JsClosure {
    pub fn new(handler: impl Fn(JsValue) -> JsValue + 'static) -> Self {
        Self {
            closure: Closure::new(|| {})
        }
    }
}