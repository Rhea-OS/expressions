#![allow(non_snake_case)]

use std::convert::Into;
use std::iter::Iterator;
use wasm_bindgen::prelude::*;
use wasm_bindgen::UnwrapThrowExt;

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
impl DataSource {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> DataSource {
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

        DataSource {
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
}

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
fn js_value_to_object(value: JsValue) -> Option<expression::Object> {
    Some(match value {
        value if value.is_string() => expression::Object::String(value.as_string()?),
        value if value.is_array() => expression::Object::List(js_sys::Array::from(&value)
            .into_iter()
            .flat_map(js_value_to_object)
            .collect()),

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
fn value_to_js_object(value: expression::Object) -> Option<JsValue> {
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
    pub fn new(config: DataSource) -> Context {
        Self {
            cx: expression::Context::new(config)
        }
    }

    #[wasm_bindgen(js_name = pushGlobal)]
    pub fn set_global(&mut self, name: js_sys::JsString, global: JsValue) {
        // Convert the global into the `expression::Object` equivalent

        self.cx.push_global(name.as_string().unwrap_throw(), js_value_to_object(global).unwrap_throw());
    }

    #[deprecated(note = "Use push_global instead")]
    #[wasm_bindgen(js_name = withGlobal)]
    pub fn with_global(self, name: js_sys::JsString, global: JsValue) -> Self {
        unimplemented!("Use the `pushGlobal` function instead`")
    }

    #[wasm_bindgen(js_name = pushOperator)]
    pub fn set_operator(&mut self, name: js_sys::JsString, global: JsValue) {
        // Convert the global into the `expression::Object` equivalent

        self.cx.push_global(name.as_string().unwrap_throw(), js_value_to_object(global).unwrap_throw());
    }

    #[deprecated(note = "Use push_operator instead")]
    #[wasm_bindgen(js_name = withOperator)]
    pub fn with_operator(self, name: js_sys::JsString, global: JsValue) -> Self {
        unimplemented!("Use the `pushOperator` function instead");
    }

    #[wasm_bindgen(js_name = evaluate)]
    pub fn evaluate(&self, expression: js_sys::JsString) -> JsValue {
        value_to_js_object(self.cx.evaluate(expression.as_string().unwrap_throw()).unwrap_throw()).unwrap_throw()
    }
}