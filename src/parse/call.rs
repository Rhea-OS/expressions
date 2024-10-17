use alloc::vec::Vec;
use nom::IResult;
use crate::parse::key::Key;
use crate::parse::{parser, ParseContext};
use crate::parse::value::{value_parser, Value};

#[derive(Debug, PartialEq)]
pub(crate) struct Call {
    pub(crate) name: Key,
    pub(crate) arguments: Vec<Value>
}

impl Call {
    pub(super) fn parse(input: &str, cx: ParseContext) -> IResult<&str, Self> {
        parser::map(parser::tuple((
            Key::parse,
            parser::delimited(parser::char('('), parser::separated_list0(parser::char(','), value_parser(cx.clone())), parser::char(')')),
        )), |(name, arguments)| Call {
            name, arguments,
        })(input)
    }
}
