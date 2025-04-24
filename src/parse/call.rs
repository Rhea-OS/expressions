use alloc::boxed::Box;
use alloc::vec::Vec;
use nom::IResult;
use crate::{
    parse::parser,
    parse::ParseContext,
    parse::value::value_parser,
    parse::value::Value
};
use crate::parse::literal::Literal;

#[derive(Debug, PartialEq)]
pub struct Call {
    pub name: Box<Value>,
    pub arguments: Vec<Value>
}

impl Call {
    pub(super) fn parse(input: &str, cx: ParseContext) -> IResult<&str, Self> {
        parser::map(parser::tuple((
            parser::alt((
                parser::delimited(parser::char('('), value_parser(cx.clone()), parser::char(')')),
                parser::map(Literal::parse, Value::Literal),
            )),
            // value_parser(cx.clone()),
            parser::delimited(parser::char('('), parser::separated_list0(parser::char(','), value_parser(cx.clone())), parser::char(')')),
        )), |(name, arguments)| Call {
            name: Box::new(name), arguments,
        })(input)
    }
}