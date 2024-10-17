use expression::{Context, DataSource, Row};
use expression::eval::Object;

/// This struct implements a friendly way to represent a row of a table.
/// The `Row` trait marks this structure with the necessary methods to be useful
struct ExampleRow {
    col1: String,
    col2: f64
}

impl Row for ExampleRow {
    fn fields(&self) -> impl Iterator<Item=&str> + Clone {
        vec!["col1", "col2"].into_iter()
    }

    fn get(&self, field: &str) -> Option<Object> {
        match field {
            "col1" => Some(Object::String(self.col1.clone())),
            "col2" => Some(Object::Number(self.col2)),
            _ => None
        }
    }
}

struct ExampleProvider<Rows: Row> {
    columns: Vec<String>,
    rows: Vec<Rows>,
}

impl<Rows: Row> DataSource for ExampleProvider<Rows> {
    type Rows = Rows;

    fn list_columns(&self) -> impl Iterator<Item=impl AsRef<str>> {
        self.columns.clone().into_iter()
    }

    fn rows(&self) -> impl Iterator<Item=&Self::Rows> {
        self.rows.iter()
    }

    fn row_mut(&mut self, row: usize) -> Option<&mut Self::Rows> {
        self.rows.get_mut(row)
    }

    fn num_rows(&self) -> usize {
        self.rows.len()
    }
}

pub fn main() {
    let cx = Context::new(ExampleProvider::<ExampleRow> {
        columns: vec![
            "col1".to_owned(),
            "col2".to_owned(),
        ],
        rows: vec![]
    });
    
    if let Ok(result) = cx.evaluate(r#"1+2"#) {
        println!("{}", result);
    }
}