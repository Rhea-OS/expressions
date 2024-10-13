#![feature(iterator_try_reduce)]
#![feature(assert_matches)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

pub mod error;
pub mod parse;
pub mod eval;
mod vec;

pub use crate::error::*;
use crate::eval::Object;
pub use crate::eval::context::Context;

pub trait Row {
    fn fields(&self) -> impl Iterator<Item=&str> + Clone;

    fn get(&self, field: &str) -> Option<Object>;
}

pub trait DataSource {
    type Rows: Row;

    fn list_columns(&self) -> impl Iterator<Item=impl AsRef<str>>;
    fn tuples(&self) -> impl Iterator<Item=&Self::Rows>;
    fn tuples_mut(&mut self) -> impl Iterator<Item=&mut Self::Rows>;

    fn num_rows(&self) -> usize;
}