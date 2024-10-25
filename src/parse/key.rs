use alloc::string::String;
use core::num::ParseIntError;
use nom::{
    error::{FromExternalError, ParseError},
    IResult
};
use crate::parse::parser;

#[derive(Debug, Eq, PartialEq)]
pub enum Key {
    Name(String),
    String(String),
}

impl Key {
    pub(super) fn parse(input: &str) -> IResult<&str, Self> {
        parser::alt((
            parser::map(parse_name, Key::Name),
            parser::map(parse_string, Key::String),
        ))(input)
    }
}

fn parse_name(input: &str) -> IResult<&str, String> {
    let mut str = String::with_capacity(input.len());

    let mut iter = input.chars();

    if let Some(c) = iter.next() {
        if nom_unicode::is_alphabetic(c) || c == '_' || c == '$' {
            str.push(c);
        } else {
            return Err(nom::Err::Error(nom::error::Error { input, code: nom::error::ErrorKind::NonEmpty }))
        }
    }

    while let Some(c) = iter.next() {
        if nom_unicode::is_alphanumeric(c) || c == '_' || c == '$' {
            str.push(c);
        } else {
            break;
        }
    }

    Ok((&input[str.len()..], str))
}

fn parse_unicode<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let parse_hex = parser::take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = parser::preceded(
        parser::char('u'),
        parser::delimited(parser::char('{'), parse_hex, parser::char('}')),
    );

    let parse_u32 = parser::map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16));

    parser::map_opt(parse_u32, core::char::from_u32)(input)
}

fn parse_escaped_char<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    parser::preceded(
        parser::char('\\'),
        parser::alt((
            parse_unicode,
            parser::value('\n', parser::char('n')),
            parser::value('\r', parser::char('r')),
            parser::value('\t', parser::char('t')),
            parser::value('\u{08}', parser::char('b')),
            parser::value('\u{0C}', parser::char('f')),
            parser::value('\\', parser::char('\\')),
            parser::value('/', parser::char('/')),
            parser::value('"', parser::char('"')),
        )),
    )(input)
}

fn parse_escaped_whitespace<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    parser::preceded(parser::char('\\'), parser::multispace1)(input)
}

fn parse_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    let not_quote_slash = parser::is_not("\"\\");

    parser::verify(not_quote_slash, |s: &str| !s.is_empty())(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

fn parse_fragment<'a, E>(input: &'a str) -> IResult<&'a str, StringFragment<'a>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    parser::alt((
        parser::map(parse_literal, StringFragment::Literal),
        parser::map(parse_escaped_char, StringFragment::EscapedChar),
        parser::value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ))(input)
}

fn parse_string<'a, E>(input: &'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let build_string = nom::multi::fold_many0(parse_fragment, String::new, |mut string, fragment| match fragment{
        StringFragment::Literal(lit) => {
            string.push_str(lit);
            string
        },
        StringFragment::EscapedChar(c) => {
            string.push(c);
            string
        },
        StringFragment::EscapedWS => {
            string
        }
    });

    parser::delimited(parser::char('"'), build_string, parser::char('"'))(input)
}