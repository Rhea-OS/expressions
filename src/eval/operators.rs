use crate::vec::Acc;
use crate::{
    error::*,
    eval::context::Operator,
    eval::context::OperatorBuilder,
    eval::Object,
};
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::{format, string::ToString, vec, vec::Vec};
use core::ops::Add;

pub(crate) fn get_standard_operators() -> Vec<Operator> {
    vec![
        OperatorBuilder::new()
            .symbol("==")
            .handler(compare)
            .build(),

        OperatorBuilder::new()
            .symbol("!=")
            .handler(inv_compare)
            .build(),

        OperatorBuilder::new()
            .symbol("&&")
            .handler(and)
            .build(),

        OperatorBuilder::new()
            .symbol("||")
            .handler(or)
            .build(),

        OperatorBuilder::new()
            .symbol("!")
            .handler(not)
            .build(),

        OperatorBuilder::new()
            .symbol(">")
            .handler(greater)
            .build(),

        OperatorBuilder::new()
            .symbol("<")
            .handler(less)
            .build(),

        OperatorBuilder::new()
            .symbol("+")
            .handler(add)
            .build(),

        OperatorBuilder::new()
            .symbol("-")
            .handler(subtract)
            .build(),

        OperatorBuilder::new()
            .symbol("*")
            .handler(multiply)
            .build(),

        OperatorBuilder::new()
            .symbol("/")
            .handler(divide)
            .build(),

        OperatorBuilder::new()
            .symbol("%")
            .handler(modulo)
            .build(),

        OperatorBuilder::new()
            .symbol("^")
            .handler(exponent)
            .build(),
    ]
}

pub fn compare(args: &[Object]) -> Result<Object> {
    if let Some((first, remaining)) = args.split_first() {
        let mut result = first.clone();

        for arg in remaining {
            result = match (result, arg) {
                (Object::Nothing, Object::Nothing) => Object::Boolean(true),
                (Object::Boolean(l), Object::Boolean(r)) => Object::Boolean(l == *r),
                (Object::Number(l), Object::Number(r)) => Object::Boolean(l == *r),
                (Object::String(l), Object::String(r)) => Object::Boolean(l.eq(r)),
                (Object::List(l), Object::List(r)) => Object::Boolean(l.eq(r)),
                (Object::AssociativeArray(l), Object::AssociativeArray(r)) => Object::Boolean(l.eq(r)),
                _ => Object::Boolean(false)
            };
        }

        Ok(result)
    } else {
        Err(ManualError::InsufficientOperands("Comparison requires at least two operands".to_owned()).into())
    }
}

pub fn inv_compare(args: &[Object]) -> Result<Object> {
    compare(args).map(|i| if let Object::Boolean(b) = i {
        Object::Boolean(!b)
    } else {
        Object::Boolean(true)
    })
}

pub fn and(args: &[Object]) -> Result<Object> {
    if let Some((first, remaining)) = args.split_first() {
        let mut result = first.clone();

        for arg in remaining {
            result = match (result, arg) {
                (Object::Boolean(l), Object::Boolean(r)) => Object::Boolean(l && *r),
                _ => return Err(ManualError::OperationNotValidForType(format!("Attempt to and {} with {}", arg.datatype(), first.datatype())).into())
            };
        }

        Ok(result)
    } else {
        Err(ManualError::InsufficientOperands("And requires at least two operands".to_owned()).into())
    }
}

pub fn or(args: &[Object]) -> Result<Object> {
    if let Some((first, remaining)) = args.split_first() {
        let mut result = first.clone();

        for arg in remaining {
            result = match (result, arg) {
                (Object::Boolean(l), Object::Boolean(r)) => Object::Boolean(l || *r),
                _ => return Err(ManualError::OperationNotValidForType(format!("Attempt to or {} with {}", arg.datatype(), first.datatype())).into())
            };
        }

        Ok(result)
    } else {
        Err(ManualError::InsufficientOperands("Or requires at least two operands".to_owned()).into())
    }
}

pub fn not(args: &[Object]) -> Result<Object> {
    if args.len() == 1 {
        if let Some(Object::Boolean(b)) = args.get(0) {
            return Ok(Object::Boolean(!b))
        }
    }

    Err(ManualError::InsufficientOperands("Not requires exactly one argument".to_owned()).into())
}

pub fn greater(args: &[Object]) -> Result<Object> {
    if let Some((first, remaining)) = args.split_first() {
        let mut result = first.clone();

        for arg in remaining {
            result = match (result, arg) {
                (Object::Number(l), Object::Number(r)) => Object::Boolean(l > *r),
                (Object::String(l), Object::String(r)) => Object::Boolean(l > *r),
                _ => return Err(ManualError::OperationNotValidForType(format!("Attempt to add {} to {}", arg.datatype(), first.datatype())).into())
            };
        }

        Ok(result)
    } else {
        Err(ManualError::InsufficientOperands("Addition requires at least two operands".to_owned()).into())
    }
}

pub fn less(args: &[Object]) -> Result<Object> {
    if let Some((first, remaining)) = args.split_first() {
        let mut result = first.clone();

        for arg in remaining {
            result = match (result, arg) {
                (Object::Number(l), Object::Number(r)) => Object::Boolean(l < *r),
                (Object::String(l), Object::String(r)) => Object::Boolean(l < *r),
                _ => return Err(ManualError::OperationNotValidForType(format!("Attempt to add {} to {}", arg.datatype(), first.datatype())).into())
            };
        }

        Ok(result)
    } else {
        Err(ManualError::InsufficientOperands("Addition requires at least two operands".to_owned()).into())
    }
}

pub fn add(args: &[Object]) -> Result<Object> {
    if let Some((first, remaining)) = args.split_first() {
        let mut result = first.clone();

        for arg in remaining {
            result = match (result, arg) {
                (Object::Number(l), Object::Number(r)) => Object::Number(l + r),
                (Object::String(l), Object::String(r)) => Object::String(String::new().add(&l).add(r)),
                (Object::List(l), Object::List(r)) => Object::List(l.clone().into_iter().chain(r.clone().into_iter()).collect()),
                (Object::List(l), r) => Object::List(l.clone().acc(r.clone())),
                (Object::AssociativeArray(l), Object::AssociativeArray(r)) => Object::AssociativeArray(l.clone().into_iter().chain(r.clone().into_iter()).collect()),
                _ => return Err(ManualError::OperationNotValidForType(format!("Attempt to add {} to {}", arg.datatype(), first.datatype())).into())
            };
        }

        Ok(result)
    } else {
        Err(ManualError::InsufficientOperands("Addition requires at least two operands".to_owned()).into())
    }
}

pub fn subtract(args: &[Object]) -> Result<Object> {
    if let Some((first, remaining)) = args.split_first() {
        let mut result = first.clone();

        for arg in remaining {
            result = match (result, arg) {
                (Object::Number(l), Object::Number(r)) => Object::Number(l - r),
                _ => return Err(ManualError::OperationNotValidForType(format!("Attempt to subtract {} from {}", arg.datatype(), first.datatype())).into())
            };
        }

        Ok(result)
    } else {
        Err(ManualError::InsufficientOperands("Subtraction requires at least two operands".to_owned()).into())
    }
}

pub fn multiply(args: &[Object]) -> Result<Object> {
    if let Some((first, remaining)) = args.split_first() {
        let mut result = first.clone();

        for arg in remaining {
            result = match (result, arg) {
                (Object::Number(l), Object::Number(r)) => Object::Number(l * r),
                _ => return Err(ManualError::OperationNotValidForType(format!("Attempt to multiply {} by {}", arg.datatype(), first.datatype())).into())
            };
        }

        Ok(result)
    } else {
        Err(ManualError::InsufficientOperands("Multiplication requires at least two operands".to_owned()).into())
    }
}

pub fn divide(args: &[Object]) -> Result<Object> {
    if let Some((first, remaining)) = args.split_first() {
        let mut result = first.clone();

        for arg in remaining {
            result = match (result, arg) {
                (Object::Number(l), Object::Number(r)) => Object::Number(l / r),
                _ => return Err(ManualError::OperationNotValidForType(format!("Attempt to divide {} by {}", arg.datatype(), first.datatype())).into())
            };
        }

        Ok(result)
    } else {
        Err(ManualError::InsufficientOperands("Division requires at least two operands".to_owned()).into())
    }
}

pub fn modulo(args: &[Object]) -> Result<Object> {
    if let Some((first, remaining)) = args.split_first() {
        let mut result = first.clone();

        for arg in remaining {
            result = match (result, arg) {
                (Object::Number(l), Object::Number(r)) => Object::Number(l % r),
                _ => return Err(ManualError::OperationNotValidForType(format!("Attempt to modulo {} by {}", arg.datatype(), first.datatype())).into())
            };
        }

        Ok(result)
    } else {
        Err(ManualError::InsufficientOperands("Modulus requires at least two operands".to_owned()).into())
    }
}

pub fn exponent(args: &[Object]) -> Result<Object> {
    if let Some((first, remaining)) = args.split_first() {
        let mut result = first.clone();

        for arg in remaining {
            result = match (result, arg) {
                (Object::Number(l), Object::Number(r)) => Object::Number(l.powf(*r)),
                _ => return Err(ManualError::OperationNotValidForType(format!("Attempt to raise {} by {}th power", arg.datatype(), first.datatype())).into())
            };
        }

        Ok(result)
    } else {
        Err(ManualError::InsufficientOperands("Exponentiation requires at least two operands".to_owned()).into())
    }
}