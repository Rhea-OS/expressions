use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use nom::{Finish, IResult};
use nom::error::ErrorKind::Many;
use crate::error::*;

pub struct AST<TokenType: Token> {
    raw: String,
    offset: usize,
    token: TokenType,
}

trait Token {
    fn parse(input: &str) -> IResult<Self, &str>;
}

struct Expression {}

struct Call {
    name: String,
    arguments: Vec<Expression>
}

enum Literal {
    Name(String),
    Number(f64),
    String(String),
    Address(String)
}

struct List {
    items: Vec<Expression>
}

struct AssociateArray {
    items: Vec<(Key, Expression)>
}

enum Key {
    Name(String),
    String(String)
}

pub fn parse(str: impl AsRef<str>) -> Result<AST<Expression>> {
    if let Ok((expr, _)) = Expression::parse(str.as_ref()).finish() {
        Ok(AST {
            raw: str.as_ref().to_owned(),
            offset: 0,
            token: expr
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

impl Token for Expression {
    fn parse(str: &str) -> IResult<Self, &str> {
        // Group the operators by precedence into a BTreeMap so it's sorted.
        let operators = OPERATORS.iter()
            .fold(BTreeMap::new(), |mut accumulator, (token, precedence, operands)| {
                if !accumulator.contains_key(precedence) {
                    accumulator.insert(*precedence, vec![]);
                }

                accumulator.get_mut(precedence).unwrap().push(token.clone());

                return accumulator;
            });

        let precedences = operators.keys().copied().collect::<Vec<_>>();

        fn expr(p: usize) -> fn(&str) -> IResult<Expression, &str> {
            if let Some(precedence) = precedences.get(p) {
                // expr(p=p+1) [operators[p]] expr(p=p) | expr(p+1)
                |input: &str| -> IResult<Expression, &str> {
                    nom::sequence::tuple(expr(p + 1), )(input)
                }
            } else {
                // ( expr(p=0) ) | Literal | Call | List | AssociativeArray
            }
        }

        expr(0)(str)
    }
}

impl Token for Literal {
    fn parse(input: &str) -> IResult<Self, &str> {
        todo!()
    }
}