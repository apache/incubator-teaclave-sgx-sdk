use std::prelude::v1::*;
use std::error;
use std::fmt::{self, Display};

use serde;

/// Error when a `Serializer` or `Deserializer` trait object fails.
#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.msg.fmt(formatter)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error {
            msg: msg.to_string(),
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error {
            msg: msg.to_string(),
        }
    }
}
