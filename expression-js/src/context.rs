use std::rc::Rc;
use wasm_bindgen::{
    JsValue,
    UnwrapThrowExt,
    prelude::*
};
use expression::error::*;
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
        value if value.is_function() => {
            let value = value.clone();
            expression::Object::Function(Rc::new(Box::new(move |args| {
                let args = js_sys::Array::from_iter(args.into_iter()
                    .map(value_to_js_object)
                    .map(|i| i.ok_or(Into::<Error>::into(ManualError::ConversionFailed)))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter());
                let result = js_sys::Function::from(value.clone()).call1(&JsValue::null(), &JsValue::from(args))
                    .unwrap_throw();
                js_value_to_object(result)
                    .ok_or(ManualError::ConversionFailed.into())
            })))
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
impl Context {
    #[wasm_bindgen(constructor)]
    pub fn new(config: DataSource) -> crate::Context {
        Self {
            cx: expression::Context::new(config)
        }
    }

    #[wasm_bindgen(js_name = pushGlobal)]
    pub fn set_global(&mut self, name: js_sys::JsString, global: JsValue) {
        // Convert the global into the `expression::Object` equivalent

        let Some(name) = name.as_string() else {
            wasm_bindgen::throw_str("Name could not be cast to native string");
        };

        let Some(obj) = js_value_to_object(global) else {
            wasm_bindgen::throw_str("Object could not be cast to target");
        };

        self.cx.push_global(name, obj);
    }

    #[deprecated(note = "Use push_global instead")]
    #[wasm_bindgen(js_name = withGlobal)]
    pub fn with_global(self, name: js_sys::JsString, global: JsValue) -> Self {
        unimplemented!("Use the `pushGlobal` function instead`")
    }

    #[wasm_bindgen(js_name = pushOperator)]
    pub fn set_operator(&mut self, operator: crate::Operator) {
        self.cx.push_operator(operator.into_operator());
    }

    #[deprecated(note = "Use push_operator instead")]
    #[wasm_bindgen(js_name = withOperator)]
    pub fn with_operator(self, name: js_sys::JsString, global: JsValue) -> Self {
        unimplemented!("Use the `pushOperator` function instead");
    }

    #[wasm_bindgen(js_name = evaluate)]
    pub fn evaluate(&self, expression: js_sys::JsString) -> JsValue {
        let Some(expr) = expression.as_string() else {
            wasm_bindgen::throw_str("Expression could not be cast to native string");
        };

        let error_message_or_result = self.cx.evaluate(expr)
            .map_err(|error| match error.into_inner() {
                global::Inner::ManualError(ManualError::InsufficientOperands(op)) => format!("The operator '{}' did not receive the required number of operands.", op),
                global::Inner::ManualError(ManualError::CannotCallNonFunctionObject()) => format!("Object not callable"),
                global::Inner::ManualError(ManualError::NoSuchOperator(op)) => format!("The operator '{}' was not recognised", op),
                global::Inner::ManualError(ManualError::NoSuchValue(value)) => format!("'{}' is not defined", value),
                global::Inner::ManualError(ManualError::OperationNotValidForType(op)) => format!("The operation '{}' was attempted on an invalid type", op),
                err => format!("Miscellaneous Error: {:?}", err)
            });

        let res = match error_message_or_result {
            Ok(result) => result,
            Err(err) => wasm_bindgen::throw_str(&err)
        };

        if let Some(res) = value_to_js_object(res) {
            res
        } else {
            wasm_bindgen::throw_str("Unable to convert result back into JS");
        }
    }
}