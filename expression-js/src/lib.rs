#![allow(non_snake_case)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DataSourceConfig {
    listColumns: js_sys::Function,
    listRows: js_sys::Function,
    getRow: js_sys::Function,
    countRows: js_sys::Function,
}

#[wasm_bindgen]
pub struct DataSource {
    config: DataSourceConfig,
    columns: Vec<String>
}

#[wasm_bindgen]
impl DataSource {
    #[wasm_bindgen(constructor)]
    pub fn new(config: DataSourceConfig) -> DataSource {
        DataSource {
            columns: vec![],
            config,
        }
    }
}

#[wasm_bindgen]
struct RowWrapper(js_sys::Object);

impl expression::Row for RowWrapper {
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
    type Rows = RowWrapper;

    fn list_columns(&self) -> impl Iterator<Item=impl AsRef<str>> {
        todo!()
    }

    fn rows(&self) -> impl Iterator<Item=&Self::Rows> {
        todo!()
    }

    fn row_mut(&mut self, row: usize) -> Option<&mut Self::Rows> {
        todo!()
    }

    fn num_rows(&self) -> usize {
        todo!()
    }
}