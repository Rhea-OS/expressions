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
    pub fn column(&self) -> Option<u32> {
        match &self.0.column {
            Column::Number(col) => {
                let mut col_number = 0;
                for char in col.chars().map(|i| i.to_ascii_lowercase()).filter(|i| i.is_ascii_alphabetic()) {
                    col_number = 10 * col_number + (char as u8 - 97) as u32;
                }
                Some(col_number)
            }

            Column::Name(_) => None
        }
    }

    #[wasm_bindgen(getter)]
    pub fn column_name(&self) -> Option<String> {
        match &self.0.column {
            Column::Number(_) => None,
            Column::Name(name) => Some(name.clone()),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn row(&self) -> Option<u32> {
        self.0.row.as_ref().and_then(|i| i.parse().ok())
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
        listColumns: () => string[],
        listRows: () => Record<string, any>[],
        getRow: (row: number) => Record<string, any>,
        countRows: () => number[],
        query: (cx: Cx, address: Address) => any | null
    };
"#;

#[wasm_bindgen(js_name=DataSource)]
pub struct DataSource {
    list_columns: js_sys::Function,
    list_rows: js_sys::Function,
    get_row: js_sys::Function,
    count_rows: js_sys::Function,
    columns: Vec<String>,
    query: js_sys::Function,

    pub(crate) cx: Rc<WasmRefCell<Object>>
}

#[wasm_bindgen(js_class=DataSource)]
impl DataSource {
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

        let query = js_sys::Function::from(js_sys::Reflect::get(&config, &JsValue::from_str("query"))
            .unwrap_throw());

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
    type Rows = RowWrapper;

    fn list_columns(&self) -> impl Iterator<Item=impl AsRef<str>> {
        self.columns.iter()
    }

    fn rows(&self) -> impl Iterator<Item=Self::Rows> {
        let rows = self.list_rows.call0(&JsValue::null())
            .unwrap_throw();

        let arr = js_sys::Array::from(&rows);

        arr.into_iter()
            .map(|i| RowWrapper(js_sys::Object::from(i)))
    }

    fn row(&self, row: usize) -> Option<Self::Rows> {
        let rows = self.get_row.call1(&JsValue::null(), &JsValue::from(row as i64))
            .unwrap_throw();

        if rows.is_falsy() {
            None
        } else {
            Some(RowWrapper(js_sys::Object::from(rows)))
        }
    }

    fn num_rows(&self) -> usize {
        let rows = self.count_rows.call0(&JsValue::null())
            .unwrap_throw();

        rows.as_f64().unwrap_or_default() as usize
    }

    fn query(&self, addr: expression::Address) -> expression::Result<Object> {
        let cx = value_to_js_object(self.cx.borrow().clone())
            .unwrap_or(JsValue::null());
        match self.query.call2(&JsValue::null(), &cx, &JsValue::from(Address::from(addr))) {
            Ok(value) => js_value_to_object(value)
                .ok_or(ManualError::ConversionFailed.into()),
            
            Err(e) => wasm_bindgen::throw_val(e),
        }
    }
}