# expressions

Parser for the expression language used in the spreadsheet plugin. This document outlines the Rust portion of the
library. For JavaScript see [the JS bindings](expression-js/README.md).

## Quickstart

In order to evaluate expressions, three things must happen;

1. A context object containing a [Data Provider](#Data Provider) must be registered
2. The list of all functions, variables and operators needs to be defined
3. The expression needs to be passed into the parser.

## Data Provider

A data provider is an object which converts addresses into values. Addresses are arbitrary text tokens wrapped in
braces. The provider determines their meaning. [See the `repl.rs`](examples/repl.rs) for a feature-complete REPL.

```rust
#[derive(Copy, Clone)]
struct Provider;

impl expression::DataSource for Provider {
    fn query(&self, query: impl AsRef<str>) -> Option<expression::Object> {
        // Parse the query however you need to.
        // For example, the format `column:row`

        Some(Object::string("Hey!"))
    }
}
```

## Context

Next you'll need a context object which holds state and variables and acts as an API to the expression engine. You can
define your own functions, globals and operators here.

```rust
fn main() {
    let mut cx = expression::Context::new(Provider)
        .with_global("twelve", expression::Object::Number(12))
        .with_fn("print", |_cx, args| {
            println!("{:?}", args);
            Ok(expression::Object::Nothing)
        });

    assert_eq!(cx.evaluate("twelve"), expression::Object::Number(12));
}
```