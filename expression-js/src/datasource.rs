use std::rc::Rc;
use wasm_bindgen::{
    JsValue,
    UnwrapThrowExt,
    prelude::*
};
use wasm_bindgen::__rt::WasmRefCell;
use wasm_bindgen::convert::{FromWasmAbi, IntoWasmAbi};
use expression::{Column, ManualError, Object};
use crate::context::{js_value_to_object, value_to_js_object};

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
    query: js_sys::Function,

    pub(crate) cx: Rc<WasmRefCell<Object>>
}

#[wasm_bindgen(js_class=DataSource)]
impl DataSource {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> crate::DataSource {
        let query = js_sys::Function::from(js_sys::Reflect::get(&config, &JsValue::from_str("query"))
            .unwrap_throw());

        crate::DataSource {
            query,

            cx: Rc::new(WasmRefCell::new(Object::Nothing))
        }
    }
}

#[wasm_bindgen]
pub struct RowWrapper(js_sys::Object);

impl expression::Row for crate::RowWrapper {
    #[allow(refining_impl_trait)]
    fn fields(&self) -> impl Iterator<Item=String> + Clone {
        js_sys::Object::keys(&self.0).into_iter()
            .filter(|i| i.is_string())
            .flat_map(|v| v.as_string())
    }

    fn get(&self, field: &str) -> Option<expression::Object> {
        js_sys::Reflect::get(&self.0, &JsValue::from_str(field)).ok()
            .and_then(|i| match i {
                i if i.is_string() => Some(expression::Object::String(i.as_string()?)),
                i if i.as_f64().is_some() => Some(expression::Object::Number(i.as_f64()?)),
                _ => Some(expression::Object::Nothing)
            })
    }
}

impl expression::DataSource for DataSource {
    fn query(&self, query: impl AsRef<str>) -> Option<Object> {
        match self.query.call1(&JsValue::null(), &JsValue::from_str(query.as_ref())) {
            Ok(value) => Some(js_value_to_object(value)?),
            Err(_) => None
        }
    }
}