mod test;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::error::*;
use crate::parse::parse;

pub struct Context {
    globals: HashMap<String, Object>
}

#[derive(Debug)]
pub enum Object {
    Boolean(bool),
    Number(f64),
    String(String),
    List(Vec<Object>),
    AssociativeArray(HashMap<String, Object>),
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", match self {
            Object::Boolean(v) => if *v { "true".to_owned() } else { "false".to_owned() },
            Object::Number(v) => v.to_string(),
            Object::String(v) => v.to_owned(),
            _ => "[Object object]".to_owned(),
        })
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

impl Context {
    pub fn new() -> Self {
        Self {
            globals: Default::default()
        }
    }

    pub fn with_global(mut self, name: impl AsRef<str>, global: Object) -> Self {
        self.globals.insert(name.as_ref().to_string(), global);
        self
    }

    pub fn evaluate(&mut self, expression: impl AsRef<str>) -> Result<Object> {
        let expr = parse(expression.as_ref())?;
        
        

        todo!();
    }
}