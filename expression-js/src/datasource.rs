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

#[derive(Clone)]
pub struct DataSource {
    pub(crate) inner: js_sys::Object,
    pub(crate) cx: Rc<WasmRefCell<Object>>
}

impl DataSource {
    pub(crate) fn from_js(js: js_sys::Object) -> Self {
        Self {
            inner: js,
            cx: Rc::new(WasmRefCell::new(Object::Nothing))
        }
    }
}

impl expression::DataSource for DataSource {
    fn query(&self, query: impl AsRef<str>) -> Option<Object> {
        let cx = value_to_js_object(self.cx.borrow().clone())?;

        match self.get_proxy()?.call2(&JsValue::null(), &cx, &JsValue::from_str(query.as_ref())) {
            Ok(value) => Some(js_value_to_object(value)?),
            Err(_) => None
        }
    }
}

impl DataSource {
    fn get_proxy(&self) -> Option<js_sys::Function> {
        match js_sys::Reflect::get(&self.inner, &JsValue::from_str("query")) {
            Ok(value) if value.is_function() => Some(value.into()),
            _ => None
        }
    }
}