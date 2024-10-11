extern crate core;

pub mod error;
mod test;
mod parse;

pub struct Address {
    
}

pub struct Value {
    
}

pub trait DataSource {
    fn get_value(value: Address) -> Option<Value>;
}
