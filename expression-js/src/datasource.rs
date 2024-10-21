use wasm_bindgen::{
    JsValue,
    UnwrapThrowExt,
    prelude::*
};

// TODO: Make type of parameter on DataSource::new() a `DataSourceConfig` type.
#[wasm_bindgen(typescript_custom_section)]
const DATA_SOURCE_CONFIG: &'static str = r#"
    export type DataSourceConfig = {
        listColumns: () => string[],
        listRows: () => Record<string, any>[],
        getRow: (row: number) => Record<string, any>,
        countRows: () => number[],
    };
"#;

#[wasm_bindgen(js_name=DataSource)]
pub struct DataSource {
    list_columns: js_sys::Function,
    list_rows: js_sys::Function,
    get_row: js_sys::Function,
    count_rows: js_sys::Function,
    columns: Vec<String>,
}

#[wasm_bindgen(js_class=DataSource)]
impl crate::DataSource {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> crate::DataSource {
        let list_columns = js_sys::Function::from(js_sys::Reflect::get(&config, &JsValue::from_str("listColumns"))
            .unwrap_throw());

        let list_rows = js_sys::Function::from(js_sys::Reflect::get(&config, &JsValue::from_str("listRows"))
            .unwrap_throw());

        let get_row = js_sys::Function::from(js_sys::Reflect::get(&config, &JsValue::from_str("getRow"))
            .unwrap_throw());

        let count_rows = js_sys::Function::from(js_sys::Reflect::get(&config, &JsValue::from_str("countRows"))
            .unwrap_throw());

        let rows = list_columns.call0(&JsValue::null())
            .unwrap_throw();

        let arr = js_sys::Array::from(&rows);

        let columns = arr.into_iter()
            .filter_map(|i| i.as_string())
            .collect();

        crate::DataSource {
            list_columns,
            list_rows,
            get_row,
            count_rows,
            columns,
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

impl expression::DataSource for crate::DataSource {
    type Rows = crate::RowWrapper;

    fn list_columns(&self) -> impl Iterator<Item=impl AsRef<str>> {
        self.columns.iter()
    }

    fn rows(&self) -> impl Iterator<Item=Self::Rows> {
        let rows = self.list_rows.call0(&JsValue::null())
            .unwrap_throw();

        let arr = js_sys::Array::from(&rows);

        arr.into_iter()
            .map(|i| crate::RowWrapper(js_sys::Object::from(i)))
    }

    fn row(&self, row: usize) -> Option<Self::Rows> {
        let rows = self.get_row.call1(&JsValue::null(), &JsValue::from(row as i64))
            .unwrap_throw();

        if rows.is_falsy() {
            None
        } else {
            Some(crate::RowWrapper(js_sys::Object::from(rows)))
        }
    }

    fn num_rows(&self) -> usize {
        let rows = self.count_rows.call0(&JsValue::null())
            .unwrap_throw();

        rows.as_f64().unwrap_or_default() as usize
    }
}
