
macro_rules! multi_error {
    ($name:ident($($manual:ident),*); $($err:ident = $obj:ty);*) => {
        pub mod $name {
            #[cfg(test)]
            use backtrace::Backtrace;
            use alloc::string::String;

            #[derive(Debug)]
            pub enum Inner {
                $($err($obj),)*
                $($manual),*
            }

            impl core::fmt::Display for Inner { fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { core::fmt::Debug::fmt(self, f) } }
            impl core::error::Error for Inner {}

            $(impl From<$obj> for Inner { fn from(value: $obj) -> Self { Self::$err(value) } })*

            pub struct Error {
                inner: Inner,

                #[cfg(test)]
                backtrace: Backtrace
            }

            impl Error {
                pub fn into_inner(self) -> Inner { self.inner }
            }

            impl<Err> From<Err> for Error where Err: Into<Inner> {
                fn from(err: Err) -> Self {
                    Self {
                        inner: err.into(),

                        #[cfg(test)]
                        backtrace: Backtrace::new()
                    }
                }
            }

            impl core::error::Error for Error {}
            impl core::fmt::Display for Error {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { core::fmt::Debug::fmt(self, f) }
            }

            impl alloc::fmt::Debug for Error {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    #[cfg(test)]
                    match std::env::var("RUST_BACKTRACE").as_ref().map(|i| i.as_ref()) {
                        Ok("full") => write!(f, "{:?}\n{:#?}", &self.inner, self.backtrace),
                        Ok("1") => write!(f, "{:?}\n{:?}", &self.inner, self.backtrace),
                        _ => write!(f, ""),
                    }

                    #[cfg(not(test))]
                    write!(f, "{:#?}", &self.inner)
                }
            }
        }
    }
}

multi_error! { global();
    ManualError = crate::error::ManualError;
    // IoError = std::io::Error;

    ParserError = nom::Err<nom::error::Error<String>>
}

pub type Result<T> = core::result::Result<T, Error>;

use alloc::string::String;
pub use global::Error;

/// TODO: Document the error types used throughout the expression parser below.
#[derive(Debug, Clone)]
pub enum ManualError {
    NoSuchOperator(String),
    NoSuchValue(String),
    OperationNotValidForType(String),
    CannotCallNonFunctionObject(),
    InsufficientOperands(String),
    CannotCastToString,
    ConversionFailed,
    ExpectedType(String),
    EmptyResultSet(String),
}

impl core::error::Error for ManualError {}
impl core::fmt::Display for ManualError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}