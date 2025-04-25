use wasm_bindgen::prelude::*;
use expression::ManualError;
use expression::error::*;
use crate::{
    convert::js_value_to_object,
    convert::value_to_js_object,
};

#[wasm_bindgen(js_name = Operator)]
pub struct Operator {
    builder: expression::OperatorBuilder,
}

#[wasm_bindgen(js_class = Operator)]
impl Operator {
    #[wasm_bindgen(constructor)]
    pub fn new(symbol: js_sys::JsString, handler: js_sys::Function) -> Operator {
        Operator {
            builder: expression::OperatorBuilder::new()
                .symbol(symbol.as_string()
                    .unwrap_throw())
                .handler(move |args| {
                    let args = js_sys::Array::from_iter(args.iter()
                        .cloned()
                        .map(value_to_js_object)
                        .map(|i| i.ok_or(Into::<Error>::into(ManualError::ConversionFailed)))
                        .collect::<Result<Vec<_>>>()?
                        .into_iter());
                    let args = JsValue::from(args);
                    let cx = JsValue::null();

                    let res = handler.call1(&cx, &args).unwrap_throw();
                    let res = js_value_to_object(res)
                        .ok_or(ManualError::ConversionFailed)?;

                    Ok(res)
                }),
        }
    }

    pub(crate) fn into_operator(self) -> expression::Operator {
        self.builder.build()
    }
}