use alloc::boxed::Box;
use alloc::string::String;
use nom::IResult;
use crate::parse::literal::Literal;
use crate::parse::ParseContext;
use crate::parse::value::{value_parser, Value};
use crate::parse::parser;

#[derive(Debug, PartialEq)]
pub struct Access {
    pub(crate) left: Box<Value>,
    pub(crate) member: Literal
}


impl Access {
    pub(super) fn parse(input: &str, cx: ParseContext) -> IResult<&str, Self> {
        parser::map(parser::tuple((
            parser::alt((
                parser::map(Literal::parse, Value::Literal),
                parser::delimited(
                    parser::char('('),
                    value_parser(cx.clone()),
                    parser::char(')'),
                )
            )),
            parser::char('.'),
            Literal::parse
        )), |(obj, _, name)| Self {
            left: Box::new(obj),
            member: name
        })(input)
    }
}