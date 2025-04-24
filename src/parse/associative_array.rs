use alloc::vec::Vec;
use nom::IResult;
use crate::{
    parse::key::*,
    parse::value::Value
};
use crate::parse::{parser, ParseContext};
use crate::parse::value::value_parser;

#[derive(Debug, PartialEq)]
pub struct AssociativeArray {
    pub items: Vec<(Key, Value)>
}

impl AssociativeArray {
    pub(super) fn parse(input: &str, cx: ParseContext) -> IResult<&str, Self> {
        parser::map(parser::delimited(parser::char('['), parser::separated_list0(parser::char(','), parser::tuple((Key::parse, parser::char('='), value_parser(cx)))), parser::char(']')), |items| AssociativeArray {
            items: items.into_iter()
                .map(|(key, _, value)| (key, value))
                .collect()
        })(input)
    }
}