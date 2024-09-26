use std::collections::{BTreeMap, HashMap};
use nom::{Finish, IResult};
use crate::error::*;

pub struct AST {
    raw: String,
    offset: usize,
    token: Token,
}

enum Token {
    Expression(Expression),
    Call(Call),
}

struct Expression {}

struct Call {}

pub fn parse(str: impl AsRef<str>) -> Result<AST> {
    if let Ok((expr, _)) = expression(str.as_ref()).finish() {
        Ok(AST {
            raw: str.as_ref().to_owned(),
            offset: 0,
            token: Token::Expression(expr),
        })
    } else {
        Err(Error::ParseError(str.as_ref().to_owned()))
    }
}

/// Operators is a static map of ("Token", "Precedence", "NumOperands")
static OPERATORS: &'static [(&'static str, i64, u64)] = &[
    ("==", 1, 2),
    ("!=", 1, 2),
    ("&&", 3, 2),
    ("||", 3, 2),
    ("!", 3, 1),
    (">", 5, 2),
    ("<", 5, 2),
    ("+", 10, 2),
    ("-", 10, 2),
    ("*", 15, 2),
    ("/", 15, 2),
    ("%", 15, 2),
    ("^", 20, 2),
];

fn expression(str: &str) -> IResult<Expression, ()> {
    let operators = OPERATORS.iter()
        .fold(BTreeMap::new(), |mut accumulator, (token, precedence, operands)| {
            if !accumulator.contains_key(precedence) {
                accumulator.insert(*precedence, vec![]);
            }

            accumulator.get_mut(precedence).unwrap().push(token.clone());

            return accumulator;
        });

    todo!()

    // operators.into_iter().rev()
    //     .scan(parse_value, |accumulator, (precedence, token)| {
    //         let operation = nom::branch::alt(operators.into_iter()
    //             .map(|i| nom::bytes::complete::tag(token)));
    //
    //         return Some(accumulator);
    //     })
}

// fn parse_value(value: &str) -> IResult<Value, >