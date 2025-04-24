use expression::Context;
use expression::DataSource;
use expression::Object;
use std::fmt::Debug;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;

/// A simple REPL to demonstrate the use of the expression crate.
/// # Features:
///  - Basic expression evaluation
///  - Simple table for demonstrating addresses
///  - Demonstrate complex usage of the API
///
/// Type an expression into the REPL and press enter.
/// The following commands are available:
///     - /exit: Exit the REPL
///     - /dump: Dump the table
///     - /set <addr>: Set the value of an address
///     - /func <name> <arg1> <arg2> ...: Define a function
///
/// # Addresses:
/// Addresses are of the form `column:row` where `column` is the name of the column and `row` is the row number. Therefore, columns may contain `:`.

#[derive(Debug, Clone)]
struct Table {
    columns: Vec<String>,
    data: Vec<Object>,
}

impl DataSource for Table {
    fn query(&self, query: impl AsRef<str>) -> Option<Object> {
        let (column, row) = query.as_ref().split_at(query.as_ref().rfind(':')?);
        let col = self.columns.iter().position(|c| c == column)?;

        self.data
            .get((&row[1..]).parse::<usize>().ok()? * self.columns.len() + col)
            .map(|i| i.clone())
    }
}

impl Table {
    pub fn empty<Col: AsRef<str>>(cols: impl AsRef<[Col]>) -> Self {
        Self {
            columns: cols.as_ref().iter().map(|c| c.as_ref().to_string()).collect(),
            data: vec![],
        }
    }

    pub fn set(&mut self, addr: impl AsRef<str>, value: Object) -> Option<Object> {
        let (column, row) = addr.as_ref().split_at(addr.as_ref().rfind(':')?);
        let col = self.columns.iter().position(|c| c == column)?;

        let index = (&row[1..]).parse::<usize>().ok()? * self.columns.len() + col;
        if index >= self.data.len() {
            self.data.resize(index + 1, Object::Nothing);
        }

        self.data[index] = value.clone();

        Some(value)
    }
}

fn prompt(prompt: impl AsRef<str>) -> String {
    let mut stdin = BufReader::new(std::io::stdin());

    std::io::stderr().write_all(prompt.as_ref().as_bytes()).unwrap();
    std::io::stderr().flush().unwrap();

    let mut cmd = String::new();
    stdin.read_line(&mut cmd).unwrap();

    cmd.trim().to_string()
}

mod commands {
    use crate::prompt;
    use crate::Table;
    use expression::Context;
    use expression::Object;

    pub fn set(cx: &mut Context<Table>, addr: impl AsRef<str>) {
        let addr = addr.as_ref();

        match cx.evaluate(prompt("--> ")) {
            Ok(v) => {
                match cx.provider_mut().set(addr, v.clone()) {
                    Some(v) => eprintln!("Set {{{addr}}} to {v}"),
                    None => eprintln!("Unable to set {{{addr}}}"),
                };
            }
            Err(err) => eprintln!("{}", err),
        }
    }
    pub fn func(cx: &mut Context<Table>, func: impl AsRef<str>) {
        let bindings = func
            .as_ref()
            .split_whitespace()
            .map(str::trim)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        let (name, bindings) = bindings.split_first().unwrap();
        let bindings = bindings.to_vec(); // Create owned copy of bindings

        let body = prompt("--> ");

        cx.push_fn(name, move |mut cx, args| {
            for (a, arg) in bindings.iter().enumerate() {
                cx.push_global(arg, args.get(a).cloned().unwrap_or(Object::Nothing));
            }

            cx.evaluate(&body)
        });
    }
    pub fn global(cx: &mut Context<Table>, name: impl AsRef<str>) {
        let name = name.as_ref();

        match cx.evaluate(prompt("--> ")) {
            Ok(v) => cx.push_global(name, v),
            Err(err) => eprintln!("{}", err)
        };
    }
}

pub fn main() {
    let mut cx = Context::new(Table::empty(["a", "b", "c"]));

    loop {
        let cmd = prompt("> ");

        match cmd.trim() {
            "/exit" => break,
            "/dump" => println!("{:#?}", cx.provider()),
            cmd if cmd.starts_with("/set ") => commands::set(&mut cx, cmd[5..].trim()),
            cmd if cmd.starts_with("/func ") => commands::func(&mut cx, cmd[6..].trim()),
            cmd if cmd.starts_with("/glob ") => commands::global(&mut cx, cmd[6..].trim()),
            cmd => match cx.evaluate(cmd) {
                Ok(v) => println!("{}", v),
                Err(err) => eprintln!("{}", err),
            },
        }
    }
}
