use crate::{
    convert::js_value_to_object,
    convert::value_to_js_object
};
use expression::Object;
use std::rc::Rc;
use wasm_bindgen::{
    prelude::*,
    JsValue,
    __rt::WasmRefCell
};

#[derive(Clone)]
pub struct DataSource {
    pub(crate) inner: js_sys::Object,
    pub(crate) ephemeral_cx: Rc<WasmRefCell<Object>>
}

impl DataSource {
    pub(crate) fn from_js(js: js_sys::Object) -> Self {
        Self {
            inner: js,
            ephemeral_cx: Rc::new(WasmRefCell::new(Object::Nothing))
        }
    }
}

impl expression::DataSource for DataSource {
    fn query(&self, query: impl AsRef<str>) -> Option<Object> {
        let cx = value_to_js_object(self.ephemeral_cx.borrow().clone())?;

        match self.get_proxy()?.call2(&self.inner, &JsValue::from_str(query.as_ref()), &cx) {
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