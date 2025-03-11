use std::io::Write;
use expression::{
    Context,
    DataSource,
    eval::Object
};

struct RowInner {
    col1: String,
    col2: f64
}

struct ExampleProvider {
    values: Vec<String>
}

impl DataSource for ExampleProvider {
    fn query(&self, query: impl AsRef<str>) -> Option<Object> {
        let index = query.as_ref().parse::<usize>().ok()?;

        self.values.get(index)
            .map(|i| Object::String(i.clone()))
    }
}

pub fn main() -> std::io::Result<()> {
    let cx = Context::new(ExampleProvider {
        values: vec![]
    });

    let mut buffer = String::new();

    eprint!("> ");
    std::io::stderr().flush()?;

    while let Ok(len) = std::io::stdin().read_line(&mut buffer) {
        let program = buffer[0..len].trim();

        match cx.evaluate(program) {
            Ok(result) => println!("{:#?}", result),
            Err(err) => println!("Error: {:?}", err)
        }

        eprint!("> ");
        std::io::stderr().flush()?;
        buffer.clear();
    }

    Ok(())
}