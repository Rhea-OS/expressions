use crate::parse::key::Key;
use crate::parse::parser;
use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum Literal { // TODO: pub(crate)
    Nothing,
    Bool(bool),
    Name(String),
    Number(f64),
    String(String),
    Address(Address),
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
    pub fn parse(input: &str) -> IResult<&str, Self> { // TODO: pub(super)
        parser::alt((
            parser::map(parser::tag("nothing"), |_| Literal::Nothing),
            parser::map(parser::tag("true"), |_| Literal::Bool(true)),
            parser::map(parser::tag("false"), |_| Literal::Bool(false)),
            parse_address,
            parse_number,
            parser::map(Key::parse, Literal::from),
        ))(input)
    }
}

fn parse_number(input: &str) -> IResult<&str, Literal> {
    parser::map(
        parser::alt((
            parse_integer,
            parse_decimal,
            // These can fail fast because they have tags, however they are not used nearly as often as the above two.
            parse_hex,
            parse_oct,
            parse_bin,
            parse_scientific,
        )),
        Literal::Number,
    )(input)
}

fn negative(input: &str) -> IResult<&str, &str> {
    parser::map(parser::opt(parser::tag("-")), |tag| {
        if tag.is_some() {
            "-"
        } else {
            ""
        }
    })(input)
}

fn parse_hex(input: &str) -> IResult<&str, f64> {
    parser::map(
        parser::tuple((
            negative,
            parser::tag("0x"),
            parser::many1(parser::alt((parser::hex_digit1, parser::tag("_")))),
        )),
        |(neg, _, body)| {
            // TODO: handle parse errors properly

            let body = body
                .into_iter()
                .flat_map(|i| i.chars())
                .filter(|i| nom::character::is_hex_digit(*i as u8))
                .collect::<String>();

            i64::from_str_radix(&format!("{}{}", neg, body), 2).unwrap() as f64
        },
    )(input)
}

fn parse_oct(input: &str) -> IResult<&str, f64> {
    parser::map(
        parser::tuple((
            negative,
            parser::tag("0o"),
            parser::many1(parser::alt((parser::oct_digit1, parser::tag("_")))),
        )),
        |(neg, _, body)| {
            // TODO: handle parse errors properly

            let body = body
                .into_iter()
                .flat_map(|i| i.chars())
                .filter(|i| nom::character::is_oct_digit(*i as u8))
                .collect::<String>();

            i64::from_str_radix(&format!("{}{}", neg, body), 2).unwrap() as f64
        },
    )(input)
}

fn parse_bin(input: &str) -> IResult<&str, f64> {
    parser::map(
        parser::tuple((
            negative,
            parser::tag("0b"),
            parser::many1(parser::alt((
                parser::char('1'),
                parser::char('0'),
                parser::char('_'),
            ))),
        )),
        |(neg, _, body): (&str, &str, Vec<char>)| {
            // TODO: handle parse errors properly

            let body = neg
                .chars()
                .chain(body.into_iter())
                .filter(|i| *i != '_')
                .collect::<String>();

            i64::from_str_radix(&body, 2).unwrap() as f64
        },
    )(input)
}

fn parse_float(input: &str) -> IResult<&str, String> {
    parser::map(
        parser::tuple((
            negative,
            parser::many0(parser::alt((parser::digit1, parser::tag("_")))),
            parser::char('.'),
            parser::many1(parser::alt((parser::digit1, parser::tag("_")))),
        )),
        |(neg, integer, _, fraction)| format!("{}{}.{}", neg, integer.join(""), fraction.join("")),
    )(input)
}

fn parse_decimal(input: &str) -> IResult<&str, f64> {
    parser::map(parse_float, |float| float.parse().unwrap())(input)
}

fn parse_scientific(input: &str) -> IResult<&str, f64> {
    parser::map(
        parser::tuple((parse_float, parser::tag_no_case("e"), parse_float)),
        |(base, _, exponent)| {
            // TODO: handle parse errors properly
            let base = base.parse::<f64>().unwrap();
            let exponent = exponent.parse::<f64>().unwrap();

            base * 10.0f64.powf(exponent)
        },
    )(input)
}

fn parse_integer(input: &str) -> IResult<&str, f64> {
    parser::map(parser::tuple((negative, parser::digit1)), |(neg, num)| {
        num.parse::<i64>()
            .map(|i| {
                if neg.len() > 0 {
                    (i * -1) as f64
                } else {
                    i as f64
                }
            })
            // TODO: Handle errors properly
            .unwrap()
    })(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Address {
    pub query: String
}

#[derive(Debug, Clone, PartialEq)]
pub enum Column {
    Number(String),
    Name(String),
}

impl Address {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        parser::map(
            parser::delimited(
                parser::char('{'),
                bracket_count,
                parser::char('}'),
            ), |query| Address {
                query: query.to_owned()
            },
        )(input)
    }
}

fn parse_address(input: &str) -> IResult<&str, Literal> {
    parser::map(Address::parse, Literal::Address)(input)
}

fn bracket_count(input: &str) -> IResult<&str, String> {
    let mut bcount = 0;

    let matched = input.chars()
        .take_while(|char| {
            match *char {
                '{' => bcount += 1,
                '}' => bcount -= 1,
                _ => ()
            };
            return bcount >= 0;
        })
        .collect::<String>();

    Ok((&input[matched.len()..], matched))
}