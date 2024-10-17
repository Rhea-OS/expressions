use alloc::vec::Vec;
use crate::{
    parse::associative_array::AssociativeArray,
    parse::call::Call,
    parse::expression::Expression,
    parse::list::List,
    parse::literal::Literal,
    parse::parser,
    parse::ParseContext,
};
use core::ops::Deref;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub(crate) enum Value {
    Expression(Expression),
    Literal(Literal),
    Call(Call),
    List(List),
    AssociativeArray(AssociativeArray),
}

pub(super) fn value_parser<'a>(cx: ParseContext) -> impl Fn(&'a str) -> IResult<&'a str, Value> + 'a {
    fn expr(rank: usize, cx: &ParseContext) -> impl Fn(&str) -> IResult<&str, Value> + '_ {
        move |input| {
            if let Some(operators) = cx.precedences.get(rank)
                .and_then(|precedence| cx.operators.get(precedence))
                .map(|i| i.iter()
                    .map(|i| i.as_str())
                    .collect::<Vec<&str>>()) {
                //  expr(p=p+1) [operators[p]] expr(p=p) | expr(p+1)

                parser::alt((
                    parser::map(parser::tuple((expr(rank + 1, cx), one_of(&operators), expr(rank, cx))), Expression::build_value),
                    expr(rank + 1, cx),
                ))(input)
            } else {
                // ( expr(p=0) ) | Literal | Call | List | AssociativeArray

                parser::alt((
                    parser::delimited(parser::char('('), expr(0, cx), parser::char(')')),
                    parser::map(|input| Call::parse(input, cx.clone()), Value::Call),
                    parser::map(|input| List::parse(input, cx.clone()), Value::List),
                    parser::map(|input| AssociativeArray::parse(input, cx.clone()), Value::AssociativeArray),
                    parser::map(Literal::parse, Value::Literal),
                ))(input)
            }
        }
    }

    move |input| expr(0, &cx)(input)
}

fn one_of<'a, 'b, Iter: Deref<Target=[&'a str]> + 'a>(items: &'a Iter) -> impl Fn(&'b str) -> IResult<&'b str, &'b str> + 'a {
    // fn one_of<'a>(items: &'a [&'a str]) -> Box<dyn Fn(&'a str) -> IResult<&'a str, &str>> {
    |input| {
        let mut last_err = None;

        for &alt in items.deref().iter() {
            match nom::bytes::complete::tag(alt)(input) {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_err = Some(error);
                }
            }
        }

        Err(last_err.unwrap_or(nom::Err::Error(nom::error::Error { input, code: nom::error::ErrorKind::NonEmpty })))
    }
}