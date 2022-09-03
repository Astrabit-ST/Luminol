use std::{fmt::{self, Display}, error};
use serde::{ser, de};

/// Define a struct of possible errors that can occur during serialization or deserialization.
/// There aren't many because Marshal (surprisingly) doesn't have many error cases.
#[derive(Debug)]
pub enum Error {
    Message(String),
    IncompatibleVersion(String),
    FormatError,
}

impl ser::Error for Error {
    fn custom<T>(msg:T) -> Self where T:Display {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg:T) -> Self where T:Display {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::IncompatibleVersion(version) => f.write_fmt(format_args!("incompatible marshal format found {} expected 4.8", version)),
            Error::FormatError => f.write_str("unexpeccted marshal format error")
        }
    }
}

impl error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;