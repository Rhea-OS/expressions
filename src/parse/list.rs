use alloc::vec::Vec;
use nom::IResult;
use crate::{
    parse::ParseContext,
    parse::parser,
    parse::value::value_parser,
    parse::value::Value
};

#[derive(Debug, PartialEq)]
pub struct List {
    pub items: Vec<Value>
}

impl List {
    pub(super) fn parse(input: &str, cx: ParseContext) -> IResult<&str, Self> {
        parser::map(parser::delimited(parser::char('['), parser::separated_list0(parser::char(','), value_parser(cx)), parser::char(']')), |items| List { items })(input)
    }
}