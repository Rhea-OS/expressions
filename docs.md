# Expression Language API

## Data Sources

```rust
pub trait DataSource {
    fn get_value(addr: impl AsRef<Address>) -> Option<Value>;
}
```

A data source is any object which can produce values from addresses.

## Values

```rust
pub struct Value {
    
}
```