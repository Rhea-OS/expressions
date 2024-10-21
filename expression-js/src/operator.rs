use wasm_bindgen::prelude::*;
use crate::context::value_to_js_object;

#[wasm_bindgen(js_name = Operator)]
pub struct Operator {
    builder: expression::OperatorBuilder
}

#[wasm_bindgen(js_class = Operator)]
impl Operator {
    #[wasm_bindgen(constructor)]
    pub fn new(symbol: js_sys::JsString, handler: js_sys::Function) -> Operator {
        Operator {
            builder: expression::OperatorBuilder::new()
                .symbol(symbol.as_string()
                    .unwrap_throw())
                .handler(|args| handler.call1(&JsValue::null(), js_sys::Array::from_iter(args.iter().map(|i| value_to_js_object(i.clone()))))),
        }
    }

    fn into_operator(self) -> expression::Operator {
        self.builder.build()
    }
}