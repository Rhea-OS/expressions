use alloc::{
    string::String,
    rc::Rc,
    boxed::Box,
    borrow::ToOwned,
    format,
    vec,
    vec::Vec,
};
use crate::{
    error::*,
    ManualError,
    Object
};

pub(crate) fn to_string(obj: Object) -> Result<String> {
    Ok(match obj {
        Object::String(str) => str.clone(),
        Object::Number(number) => format!("{}", number),
        Object::Boolean(boolean) => format!("{}", boolean),
        Object::Nothing => "nothing".to_owned(),
        Object::List(ls) => format!("{}", ls.iter().cloned().map(to_string)
            .zip(core::iter::repeat(", ".to_owned()))
            .map(|(i, j)| i.map(|i| [i, j]))
            .collect::<Result<Vec<[String; 2]>>>()?
            .into_iter()
            .flatten()
            .collect::<String>()),
        Object::AssociativeArray(ls) => format!("{}", ls.iter()
            .map(|(i, j)| (i.clone(), j.clone()))
            .map(|(key, value)| Ok(format!("\n    {} = {},", key, to_string(value)?)))
            .collect::<Result<String>>()?),
        Object::Function(_) => Err(ManualError::CannotCastToString)?
    })
}

pub(crate) fn get_standard_operators() -> Vec<(String, Object)> {
    vec![("toString".to_owned(), Object::Function(Rc::new(Box::new(|args| if args.len() != 1 {
        Ok(Object::String(to_string(args.get(0).unwrap().clone())?))
    } else {
        Err(ManualError::InsufficientOperands("toString".to_owned()).into())
    }))))]
}