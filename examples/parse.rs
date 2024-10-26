use std::{
    cell::RefCell,
    rc::Rc
};
use std::io::Write;
use expression::{
    Context,
    DataSource,
    Row,
    eval::Object
};

/// This struct implements a friendly way to represent a row of a table.
/// The `Row` trait marks this structure with the necessary methods to be useful
///
/// This example uses interior mutability because `DataSource` trait works with owned types such as `Rc`s.
#[derive(Clone)]
struct ExampleRow {
    inner: Rc<RefCell<RowInner>>
}

struct RowInner {
    col1: String,
    col2: f64
}

impl Row for ExampleRow {
    fn fields(&self) -> impl Iterator<Item = impl AsRef<str>> + Clone {
        vec!["col1", "col2"].into_iter()
    }

    fn get(&self, field: &str) -> Option<Object> {
        let inner = self.inner.borrow();

        match field {
            "col1" => Some(Object::String(inner.col1.clone())),
            "col2" => Some(Object::Number(inner.col2)),
            _ => None
        }
    }
}

struct ExampleProvider<Rows: Row> {
    columns: Vec<String>,
    rows: Vec<Rows>,
}

impl<Rows: Row + Clone> DataSource for ExampleProvider<Rows> {
    type Rows = Rows;

    fn list_columns(&self) -> impl Iterator<Item=impl AsRef<str>> {
        self.columns.clone().into_iter()
    }

    fn rows(&self) -> impl Iterator<Item=Self::Rows> {
        self.rows.iter()
            .cloned()
    }

    fn row(&self, row: usize) -> Option<Self::Rows> {
        self.rows.get(row)
            .cloned()
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

    let parser = cx.parse_context();

    let program = std::env::args().nth(1).unwrap();

    print!("{}: ", &program);
    std::io::stdout().flush().unwrap();

    match parser.parse(program) {
        Ok(result) => println!("{:#?}", result),
        Err(err) => println!("Error: {:?}", err)
    }
}