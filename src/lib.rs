extern crate core;

pub mod error;
mod parse;
mod eval;

pub use crate::error::*;

pub struct Address {
    
}

pub struct Value {
    
}

pub trait DataSource {
    fn get_value(value: Address) -> Option<Value>;
}
