# expressions

Parser for the expression language used in the spreadsheet plugin

## Quickstart

In order to evaluate expressions, three things must happen;

1. A context object containing a [Data Provider](#DataProvider) must be registered
2. The list of all functions, variables and operators needs to be defined
3. The expression needs to be passed in to the parser.

## Data Provider

A data provider is an object which converts addresses into values. 
It is used as a translation layer between the tabular data and the expression.

To achieve this, you need to define
1. A provider structure
2. The row type the provider serves.

