use crate::parse::value::Value;
use alloc::{
    borrow::ToOwned,
    string::String,
    vec,
    vec::Vec,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Expression {
    pub(crate) operands: Vec<Value>,
    pub(crate) operator: String,
}

impl Expression {
    pub(super) fn build_value(expr: (Value, &str, Value)) -> Value {
        Value::Expression(Self {
            operands: vec![expr.0, expr.2],
            operator: expr.1.to_owned(),
        })
    }
}