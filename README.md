# expressions

Parser for the expression language used in the spreadsheet plugin

## Quickstart

In order to evaluate expressions, three things must happen;

1. A context object containing a [Data Provider](#Data Provider) must be registered
2. The list of all functions, variables and operators needs to be defined
3. The expression needs to be passed in to the parser.

## Data Provider

A data provider is an object which converts addresses into values. 
It is used as a translation layer between the tabular data and the expression.

To achieve this, you need to define
1. A provider structure
2. The row type the provider serves.
3. Implement the `DataSource` and `Row` traits respectively.
4. Parse the actual data source to gain knowledge of the available data.

## Example Provider

We will be constructing a provider which sits atop a CSV parser which we will not be implementing here.

### 1. CSV Parser

For demonstration purposes, we will assume that a CSV parser yields a structure which implements the following trait:

```rust
pub trait CSVDocument {
    fn get_cell(&self, row: usize, col: usize) -> &str;
    fn get_cell_mut(&mut self, row: usize, col: usize) -> &mut str;
    
    /// Iterates over all cells of a column
    fn iter_col(&self, col: usize) -> impl Iterator<Item=impl AsRef<str>>;
    /// Iterates over all cells of a row
    fn iter_row(&self, row: usize) -> impl Iterator<Item=impl AsRef<str>>;
    
    /// Iterates over all cells of a column mutably
    fn iter_col_mut(&mut self, row: usize) -> impl Iterator<Item=impl AsMut<str>>;
    /// Iterates over all cells of a row mutably
    fn iter_row_mut(&mut self, row: usize) -> impl Iterator<Item=impl AsMut<str>>;
    
    /// Iterates over all rows
    fn iter_rows(&self) -> impl Iterator<Item=Vec<impl AsRef<str>>>;
    /// Iterates over all rows mutably
    fn iter_rows_mut(&mut self) -> impl Iterator<Item=Vec<impl AsMut<str>>>;
}
```

We will assume the structure which implements this is called `CSVFile`.

```rust
#[derive(CSVDocument)]
pub struct CSVFile {
    // --- snip ---
}
```

### 2. Data Source

The provider will assume the first row contains the headers. Thus, the referenced cell will be shifted up by 1 row.

```rust
use std::collections::HashMap;
use expressions::{
    DataSource,
    Row
};

pub struct CSVRow {
    column_name_to_column_index: HashMap<String, usize>,
    values: Vec<String>
}

impl DataSource for CSVFile {
    type Rows = CSVRow;
    
    fn list_columns(&self) -> impl Iterator<Item=impl AsRef<str>> {
          self.iter_col(0)
    }
    
    fn tuples(&self) -> impl Iterator<Item=impl AsRef<CSVRow>> {
        let columns = self.iter_row(0)
            .enumerate()
            .map(|(a, i)| (i.to_owned(), a))
            .collect::<HashMap<_, _>>();
        
        self.iter_rows()
            .skip(1)
            .enumerate()
            .map(|(a, values)| CSVRow {
                column_name_to_column_index: columns.clone(),
                values
            })
    }

    fn tuples_mut(&mut self) -> impl Iterator<Item=impl AsMut<CSVRow>> {
        let columns = self.iter_row(0)
            .enumerate()
            .map(|(a, i)| (i.to_owned(), a))
            .collect::<HashMap<_, _>>();

        self.iter_rows_mut()
            .skip(1)
            .enumerate()
            .map(|(a, values)| CSVRow {
                column_name_to_column_index: columns.clone(),
                values: 
            })
    }
}
```