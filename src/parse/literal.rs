use crate::parse::key::Key;
use crate::parse::parser;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub(crate) enum Literal {
    Name(String),
    Number(f64),
    String(String),
    Address(String),
}

impl From<Key> for Literal {
    fn from(key: Key) -> Self {
        match key {
            Key::Name(name) => Self::Name(name),
            Key::String(string) => Self::String(string),
        }
    }
}

impl Literal {
    pub(super) fn parse(input: &str) -> IResult<&str, Self> {
        parser::alt((
            parse_number,
            parser::map(Key::parse, Literal::from),
            // parse_address
        ))(input)
    }
}

fn parse_number(input: &str) -> IResult<&str, Literal> {
    parser::map(parser::alt((
        parse_hex,
        parse_oct,
        parse_bin,
        parse_integer,
        parse_decimal,
        parse_scientific,
    )), Literal::Number)(input)
}

fn negative(input: &str) -> IResult<&str, &str> {
    parser::map(parser::opt(parser::tag("-")), |tag| if tag.is_some() { "-" } else { "" })(input)
}

fn parse_hex(input: &str) -> IResult<&str, f64> {
    parser::map(parser::tuple((
        negative,
        parser::tag("0x"),
        parser::many1(parser::alt((
            parser::hex_digit1,
            parser::tag("_")
        )))
    )), |(neg, _, body)| {
        // TODO: handle parse errors properly

        let body = body.into_iter()
            .flat_map(|i| i.chars())
            .filter(|i| nom::character::is_hex_digit(*i as u8))
            .collect::<String>();

        i64::from_str_radix(&format!("{}{}", neg, body), 2).unwrap() as f64
    })(input)
}

fn parse_oct(input: &str) -> IResult<&str, f64> {
    parser::map(parser::tuple((
        negative,
        parser::tag("0o"), parser::many1(parser::alt((
            parser::oct_digit1,
            parser::tag("_")
        )))
    )), |(neg, _, body)| {
        // TODO: handle parse errors properly

        let body = body.into_iter()
            .flat_map(|i| i.chars())
            .filter(|i| nom::character::is_oct_digit(*i as u8))
            .collect::<String>();

        i64::from_str_radix(&format!("{}{}", neg, body), 2).unwrap() as f64
    })(input)
}

fn parse_bin(input: &str) -> IResult<&str, f64> {
    parser::map(parser::tuple((
        negative,
        parser::tag("0b"),
        parser::many1(parser::alt((
            parser::tag("1"),
            parser::tag("0"),
            parser::tag("_")
        )))
    )), |(neg, _, body): (&str, &str, Vec<&str>)| {
        // TODO: handle parse errors properly

        let body = body.into_iter()
            .filter(|i| *i != "_")
            .collect::<String>();

        i64::from_str_radix(&format!("{}{}", neg, body), 2).unwrap() as f64
    })(input)
}

fn parse_float(input: &str) -> IResult<&str, String> {
    parser::map(parser::tuple((
        negative,
        parser::many0(parser::alt((
            parser::digit1,
            parser::tag("_")
        ))),
        parser::tag("."),
        parser::many1(parser::alt((
            parser::digit1,
            parser::tag("_")
        ))),
    )), |(neg, integer, _, fraction)| format!("{}{}.{}", neg, integer.join(""), fraction.join("")))(input)
}

fn parse_decimal(input: &str) -> IResult<&str, f64> {
    parser::map(parse_float, |float| float.parse().unwrap())(input)
}

fn parse_scientific(input: &str) -> IResult<&str, f64> {
    parser::map(parser::tuple((
        parse_float,
        parser::tag_no_case("e"),
        parse_float,
    )), |(base, _, exponent)| {
        // TODO: handle parse errors properly
        let base = base.parse::<f64>().unwrap();
        let exponent = exponent.parse::<f64>().unwrap();

        base * 10.0f64.powf(exponent)
    })(input)
}

fn parse_integer(input: &str) -> IResult<&str, f64> {
    parser::map(parser::tuple((
        negative,
        parser::digit1
    )), |(neg, num)| {
        // TODO: handle parse errors properly
        format!("{}{}", neg, num).parse().unwrap()
    })(input)
}

fn parse_address(input: &str) -> IResult<&str, Literal> {
    todo!()
}