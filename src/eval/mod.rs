mod test;
pub mod context;
pub mod operators;
mod globals;

use alloc::{string::String, string::ToString, borrow::ToOwned, vec::Vec, boxed::Box, rc::Rc, format};
use core::fmt::{Debug, Display, Formatter};
use nom::lib::std::collections::HashMap;
use crate::{Context, DataSource};
use crate::error::*;

#[derive(Clone)]
pub enum Object {
    Nothing,
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Rc<dyn Fn(Vec<Object>) -> Result<Object>>),
    List(Vec<Object>),
    AssociativeArray(HashMap<String, Object>),
}

impl Object {
    pub fn string(str: impl AsRef<str>) -> Self {
        Self::String(str.as_ref().to_owned())
    }

    pub fn function(fun: impl Fn(Vec<Object>) -> Result<Object> + 'static) -> Self {
        Self::Function(Rc::new(fun))
    }

    pub fn datatype(&self) -> &str {
        match self {
            Object::Nothing => "nothing",
            Object::Boolean(_) => "boolean",
            Object::Number(_) => "number",
            Object::String(_) => "string",
            Object::List(_) => "list",
            Object::AssociativeArray(_) => "associative_array",
            Object::Function(_) => "function",
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", match self {
            Object::Nothing => "nothing".to_owned(),
            Object::Boolean(v) => if *v { "true".to_owned() } else { "false".to_owned() },
            Object::Number(v) => v.to_string(),
            Object::String(v) => format!("'{}'", v),
            Object::Function(_) => "fn()".to_owned(),
            Object::List(list) => format!("[{}]", list.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")),
            Object::AssociativeArray(assoc) => format!("[{}]", assoc.iter().map(|(a, i)| format!("{}={}", a, i)).collect::<Vec<_>>().join(", ")),
        })
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl PartialEq<f64> for Object {
    fn eq(&self, other: &f64) -> bool {
        match self {
            Object::Number(x) => x == other,
            _ => false
        }
    }
}

impl PartialEq<&str> for Object {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Object::String(str) => str.eq(other),
            _ => false
        }
    }
}

impl PartialEq<&[Object]> for Object {
    fn eq(&self, other: &&[Object]) -> bool {
        match self {
            Object::List(list) => list == other,
            _ => false
        }
    }
}

impl PartialEq<Object> for Object {
    fn eq(&self, other: &Object) -> bool {
        match (self, other) {
            (Object::Nothing, Object::Nothing) => true,
            (Object::Number(l), Object::Number(r)) => l == r,
            (Object::String(l), Object::String(r)) => l == r,
            (Object::Boolean(l), Object::Boolean(r)) => l == r,

            (Object::Function(l), Object::Function(r)) => Rc::ptr_eq(l, r),

            (Object::List(l), Object::List(r)) => l == r,
            (Object::AssociativeArray(l), Object::AssociativeArray(r)) => l == r,

            _ => false
        }
    }
}