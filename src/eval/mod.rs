mod test;
mod context;
mod operators;

use alloc::{
    string::String,
    string::ToString,
    borrow::ToOwned,
    vec::Vec
};
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::fmt::{Debug, Display, Formatter};
use wasm_bindgen::__rt::std::collections::HashMap;
use crate::error::*;

#[derive(Clone)]
pub enum Object {
    Nothing,
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Rc<Box<dyn Fn(Vec<Object>) -> Result<Object>>>),
    List(Vec<Object>),
    AssociativeArray(HashMap<String, Object>),
}

impl Object {
    pub fn function(fun: impl Fn(Vec<Object>) -> Result<Object> + 'static) -> Self {
        Self::Function(Rc::new(Box::new(fun)))
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", match self {
            Object::Boolean(v) => if *v { "true".to_owned() } else { "false".to_owned() },
            Object::Number(v) => v.to_string(),
            Object::String(v) => v.to_owned(),
            Object::Function(fun) => "fn()".to_owned(),
            _ => "[Object object]".to_owned(),
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