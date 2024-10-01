mod key;
mod call;
mod list;
mod literal;
mod associative_array;
mod expression;
mod value;

use std::collections::BTreeMap;
use std::ops::Deref;
use std::rc::Rc;
use nom::IResult;
use crate::{
    error::*,
    parse::value::Value,
    parse::value::OPERATORS
};
use crate::parse::value::value_parser;

mod parser {
    pub use nom::branch::*;
    pub use nom::combinator::*;
    pub use nom::bytes::complete::*;
    pub use nom::sequence::tuple;
}

struct Context(Rc<ContextInner>);

impl Context {
    fn new(precedences: Vec<i64>, operators: BTreeMap<i64, Vec<&'static str>>) -> Self {
        Self(Rc::new(ContextInner {
            precedences,
            operators
        }))
    }
}

impl Deref for Context {
    type Target = ContextInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for Context  {
    fn clone(&self) -> Self {
        Context(Rc::clone(&self.0))
    }
}

struct ContextInner {
    operators: BTreeMap<i64, Vec<&'static str>>,
    precedences: Vec<i64>,
}

impl Context {
    fn get_operators_for_rank<'input, 'cx: 'input>(&'cx self, rank: usize) -> Option<impl Fn(&'input str) -> IResult<&'input str, &'input str>> {
        let operators = self.precedences.get(rank)
            .and_then(|precedence| self.operators.get(precedence))?;

        Some(|input| {
            let mut last_err = None;

            for &alt in operators.iter() {
                match nom::bytes::complete::tag(alt)(input) {
                    Ok(result) => return Ok(result),
                    Err(error) => {
                        last_err = Some(error);
                    }
                }
            }

            Err(last_err.unwrap_or(nom::Err::Error(nom::error::Error { input, code: nom::error::ErrorKind::NonEmpty })))
        })
    }
}

pub fn parse(str: impl AsRef<str>) -> Result<Value> {
    // Group the operators by precedence into a BTreeMap so it's sorted.
    let operators = OPERATORS.iter()
        .fold(BTreeMap::new(), |mut accumulator, (token, precedence, _num_operands)| {
            if !accumulator.contains_key(precedence) {
                accumulator.insert(*precedence, vec![]);
            }

            accumulator.get_mut(precedence).unwrap().push(token.clone());

            return accumulator;
        });

    value_parser(Context::new(
        operators.keys().copied().collect::<Vec<_>>(),
        operators
    ))(str.as_ref())
        .map(|(_, value)| value)
        .map_err(|_| Error::ParseError(str.as_ref().to_owned()))
}