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
// mod hashmap;

use alloc::vec::Vec;
pub use crate::error::*;
pub use crate::eval::*;
pub use crate::eval::context::*;
pub use crate::parse::literal::Address;
pub use crate::parse::literal::Column;

/// # Row
/// Represents a row of a table.
///
/// The aim of this struct is to behave like an associative array with a predefined list of columns.
/// It may be a representation of data fetched from an actual CSV file, an SQL table or any other tabular datasource.
///
/// ## Example
/// ```rust
/// use expression::eval::Object;
/// use expression::Row;
///
/// struct Example {
///     col1: f64,
///     col2: String
/// }
///
/// impl Row for Example {
///     fn fields(&self) -> impl Iterator<Item=impl AsRef<str>> + Clone {
///         // Ideally, this function returns the list of available fields in a programmatic way, but this example is too trivial.
///         vec!["col1", "col2"].clone().into_iter()
///     }
///
///     fn get(&self, field: &str) -> Option<Object> {
///         match field {
///             "col1" => Some(Object::Number(self.col1)),
///             "col2" => Some(Object::String(self.col2.clone())),
///             _ => None
///         }
///     }
/// }
/// ```
pub trait Row {
    /// Retrieves the list of columns the row contains.
    ///
    /// > **Note:** While situations may occur, where a row can have different fields than the overall table,
    /// this is not normally useful and should be treated as invalid, despite being semantically valid.
    /// > i.e. this function should return the same values **in the same order** as `DataSource::list_columns`.
    fn fields(&self) -> impl Iterator<Item=impl AsRef<str>> + Clone;

    /// Retrieve the value of a field.
    fn get(&self, field: &str) -> Option<Object>;
}

/// # Data Source
/// A datasource may be thought of as a self-contained table.
/// It may produce its values either by reading them from a data source or computing them directly.
///
/// ## Examples of where data sources are useful
/// * Building spreadsheets based on CSV files
/// * Visualising the result of SQL queries
/// * Data entry forms
pub trait DataSource {
    type Rows: Row;

    /// List the columns the table contains. Should be identical to `Self::Rows::fields()`
    fn list_columns(&self) -> impl Iterator<Item=impl AsRef<str>>;

    /// Iterates over the rows in the table
    fn rows(&self) -> impl Iterator<Item=Self::Rows>;
    //
    // /// Mutably iterates over the rows in the table
    // fn tuples_mut(&mut self) -> impl Iterator<Item=impl AsMut<Self::Rows>>;
    //
    // /// Get a mutable reference to a row
    // fn row_mut(&mut self, row: usize) -> Option<&mut Self::Rows>;

    fn row(&self, row: usize) -> Option<Self::Rows>;

    /// How many rows the table contains
    fn num_rows(&self) -> usize;
    
    fn query(&self, addr: Address) -> Result<Object>;
}

pub struct EmptyProvider;
pub struct EmptyRow;

impl Row for EmptyRow {
    fn fields(&self) -> impl Iterator<Item = impl AsRef<str>> + Clone {
        Vec::<&'static str>::new().into_iter()
    }

    fn get(&self, _field: &str) -> Option<Object> {
        None
    }
}

impl DataSource for EmptyProvider {
    type Rows = EmptyRow;

    fn list_columns(&self) -> impl Iterator<Item = impl AsRef<str>> {
        Vec::<&'static str>::new().into_iter()
    }

    fn rows(&self) -> impl Iterator<Item=Self::Rows> {
        alloc::vec![].into_iter()
    }

    fn row(&self, row: usize) -> Option<Self::Rows> {
        None
    }

    fn num_rows(&self) -> usize {
        0
    }
    
    fn query(&self, _addr: Address) -> Result<Object> {
        Ok(Object::Nothing)
    }
}

impl EmptyProvider {
    pub fn new() -> Self {
        Self
    }
}