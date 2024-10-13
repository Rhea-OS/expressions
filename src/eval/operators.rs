use alloc::{
    string::ToString,
    vec,
    vec::Vec
};
use crate::{
    eval::context::Operator,
    eval::context::OperatorBuilder,
    eval::Object,
    error::*
};

pub(crate) fn get_standard_operators() -> Vec<Operator> {
    vec![
        OperatorBuilder::new()
            .symbol("+")
            .handler(add)
            .build(),

        OperatorBuilder::new()
            .symbol("-")
            .handler(subtract)
            .build(),
    ]
}

pub fn add(args: &[Object]) -> Result<Object> {
    let mut sum = 0.0f64;

    for arg in args {
        if let Object::Number(number) = arg {
            sum += number;
        } else {
            return Err(ManualError::OperationNotValidForType("+".to_string()).into());
        }
    }

    Ok(Object::Number(sum))
}

pub fn subtract(args: &[Object]) -> Result<Object> {
    let mut sum = 0.0f64;

    for arg in args {
        if let Object::Number(number) = arg {
            sum -= number;
        } else {
            return Err(ManualError::OperationNotValidForType("-".to_string()).into());
        }
    }

    Ok(Object::Number(sum))
}