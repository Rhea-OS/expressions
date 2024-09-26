use std::fmt::Display;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::error::Error as Err;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ParseError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Err for Error {}