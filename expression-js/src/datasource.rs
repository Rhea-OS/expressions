use crate::{
    context::js_value_to_object,
    context::value_to_js_object
};
use expression::Object;
use std::rc::Rc;
use wasm_bindgen::{
    prelude::*,
    JsValue,
    UnwrapThrowExt,
    convert::FromWasmAbi,
    convert::IntoWasmAbi,
    __rt::WasmRefCell
};

#[wasm_bindgen(js_name=Address)]
pub struct Address(expression::Address);

#[wasm_bindgen(js_class = Address)]
impl Address {
    #[wasm_bindgen(constructor)]
    pub fn new(address: js_sys::JsString) -> Self {
        let Some(address) = address.as_string() else {
            wasm_bindgen::throw_str("Expected Address");
        };

        if let Ok((_, address)) = expression::Address::parse(&address) {
            Self(address)
        } else {
            wasm_bindgen::throw_str("failed to parse address");
        }
    }

    #[wasm_bindgen(getter)]
    pub fn query(&self) -> String {
        self.0.query.clone()
    }
}

impl Address {
    pub fn from(addr: expression::Address) -> Self {
        Self(addr)
    }
}


// TODO: Make type of parameter on DataSource::new() a `DataSourceConfig` type.
#[wasm_bindgen(typescript_custom_section)]
const DATA_SOURCE_CONFIG: &'static str = r#"
    export type DataSourceConfig<Cx> = {
        query: (cx: Cx, address: Address) => any | null
    };
"#;

#[wasm_bindgen(js_name=DataSource)]
pub struct DataSource {
    query_handler: js_sys::Function,

    pub(crate) cx: Rc<WasmRefCell<Object>>
}

#[wasm_bindgen(js_class=DataSource)]
impl DataSource {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> crate::DataSource {
        crate::DataSource {
            query_handler: js_sys::Function::from(js_sys::Reflect::get(&config, &JsValue::from_str("query"))
                .unwrap_throw()),

            cx: Rc::new(WasmRefCell::new(Object::Nothing))
        }
    }
}

impl expression::DataSource for DataSource {
    fn query(&self, query: impl AsRef<str>) -> Option<Object> {
        let cx = value_to_js_object(self.cx.borrow().clone())
            .unwrap_or(JsValue::null());

        match self.query_handler.call2(&JsValue::null(), &cx, &JsValue::from_str(query.as_ref())) {
            Ok(value) => Some(js_value_to_object(value)?),
            Err(_) => None
        }
    }
}