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
    Object,
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

fn global<Func: Fn(Vec<Object>) -> Result<Object> + 'static>(name: impl AsRef<str>, func: Func) -> (String, Object) {
    (name.as_ref().to_owned(), Object::Function(Rc::new(Box::new(func))))
}

fn constant(name: impl AsRef<str>, constant: f64) -> (String, Object) {
    (name.as_ref().to_owned(), Object::Number(constant))
}

mod globals {
    use alloc::borrow::ToOwned;
    use alloc::vec::Vec;
    use crate::{ManualError, Object, error::*};

    type Global = fn(Vec<Object>) -> Result<Object>;

    macro_rules! glob {
        ($($arg:ident),* => $body:expr) => {
            |args| {
                let mut args = args.into_iter();
                $(let $arg = args.next();)*

                $body
            }
        };
    }

    pub(super) const to_string: Global = |args| if args.len() != 1 {
        Ok(Object::String(crate::eval::globals::to_string(args.get(0).unwrap().clone())?))
    } else {
        Err(ManualError::InsufficientOperands("toString".to_owned()).into())
    };

    pub(super) const identity: Global = |args| args.get(0)
        .ok_or(ManualError::InsufficientOperands("identity".to_owned()).into())
        .cloned();

    pub(super) const sin: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.sin())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const cos: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.cos())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const tan: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.tan())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const sinh: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.sinh())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const cosh: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.cosh())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const tanh: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.tanh())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const asin: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.asin())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const acos: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.acos())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const atan: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.atan())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const asinh: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.asinh())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const acosh: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.acosh())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const atanh: Global = glob!(x => match x {
        Some(Object::Number(x)) => Ok(Object::Number(x.atanh())),
        _ => Err(ManualError::ExpectedType("Number".to_owned()).into())
    });

    pub(super) const atan2: Global = glob!(x, y => match (x, y) {
        (Some(Object::Number(x)), Some(Object::Number(y))) => Ok(Object::Number(x.atan2(y))),
        _ => Err(ManualError::ExpectedType("Number, Number".to_owned()).into())
    });
}

pub(crate) fn get_standard_globals() -> Vec<(String, Object)> {
    vec![
        global("toString", globals::to_string),
        global("identity", globals::identity),

        global("sin", globals::sin),
        global("cos", globals::cos),
        global("tan", globals::tan),
        global("sinh", globals::sinh),
        global("cosh", globals::cosh),
        global("tanh", globals::tanh),
        global("asin", globals::asin),
        global("acos", globals::acos),
        global("atan", globals::atan),
        global("asinh", globals::asinh),
        global("acosh", globals::acosh),
        global("atanh", globals::atanh),
        global("atan2", globals::atan2),

        constant("PI", core::f64::consts::PI),
        constant("π", core::f64::consts::PI),
        constant("e", core::f64::consts::E),
        constant("E", core::f64::consts::E),
        constant("LOG2_e", core::f64::consts::LOG2_E),
        constant("LOG2_10", core::f64::consts::LOG2_10),
        constant("LOG10_2", core::f64::consts::LOG10_2),
    ]
}