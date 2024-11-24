use alloc::format;
use crate::parse::parser;
use alloc::string::{String, ToString};
use core::num::ParseIntError;
use nom::{error::{FromExternalError, ParseError}, FindToken, IResult};

#[derive(Debug, Eq, PartialEq)]
pub enum Key {
    Name(String),
    String(String),
}

impl Key {
    pub(super) fn parse(input: &str) -> IResult<&str, Self> {
        parser::alt((
            parser::map(parse_name, Key::Name),
            parser::map(parse_string('"', '"'), Key::String),
            parser::map(parse_string('\'', '\''), Key::String),
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
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::NonEmpty,
            }));
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
            parser::value('\'', parser::char('\'')),
        )),
    )(input)
}

fn parse_escaped_whitespace<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    parser::preceded(parser::char('\\'), parser::multispace1)(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

struct Str(String);

impl FindToken<char> for Str {
    fn find_token(&self, token: char) -> bool {
        self.0.chars().any(|c| token == c)
    }
}

pub(crate) fn parse_string<'a>(start: char, end: char) -> impl Fn(&'a str) -> IResult<&'a str, String> {
    move |input| {
        let terminator = Str(format!("{}\\", end));

        let parse_literal = parser::verify(parser::is_not(terminator), |s: &str| !s.is_empty());

        let fragment = parser::alt((
            parser::map(parse_literal, StringFragment::Literal),
            parser::map(parse_escaped_char, StringFragment::EscapedChar),
            parser::value(StringFragment::EscapedWS, parse_escaped_whitespace),
        ));

        let parse_raw_string = nom::multi::fold_many0(
            fragment,
            String::new,
            |mut string, fragment| match fragment {
                StringFragment::Literal(lit) => {
                    string.push_str(lit);
                    string
                }
                StringFragment::EscapedChar(c) => {
                    string.push(c);
                    string
                }
                StringFragment::EscapedWS => string,
            },
        );

        parser::delimited(parser::char(start), parse_raw_string, parser::char(end))(input)
    }
}
