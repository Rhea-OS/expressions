#![feature(iterator_try_reduce)]
#![feature(assert_matches)]
#![feature(iter_array_chunks)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

pub mod error;
pub mod parse;
pub mod eval;
mod vec;

pub use crate::error::*;
pub use crate::eval::*;
pub use crate::eval::context::*;
pub use crate::parse::literal::Address;
pub use crate::parse::literal::Column;
pub use crate::parse::value::Value;

/// # Data Source
/// A datasource which responds to queries.
/// It may produce its values either by reading them from a data source or computing them directly.
///
/// ## Examples of where data sources are useful
/// * Building spreadsheets based on CSV files
/// * Visualising the result of SQL queries
/// * Data entry forms
pub trait DataSource {
    fn query(&self, query: impl AsRef<str>) -> Option<Object>;
}

impl<T: DataSource> DataSource for &T {
    fn query(&self, query: impl AsRef<str>) -> Option<Object> {
        T::query(self, query)
    }
}

pub struct EmptyProvider;

impl EmptyProvider {
    pub fn new() -> Self {
        Self
    }
}

impl DataSource for EmptyProvider {
    fn query(&self, _: impl AsRef<str>) -> Option<Object> {
        None
    }
}