//!error system
use std::{error::Error as StdErr, fmt, io, num, str};

#[derive(Debug, PartialEq)]
pub enum ParseErr {
    Utf8(str::Utf8Error),
    Int(num::ParseIntError),
    StatusErr,
    HeadersErr,
    UriErr,
    Invalid,
    Empty,
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseErr: {}", self.description())
    }
}

impl StdErr for ParseErr {
    fn description(&self) -> &str {
        use self::ParseErr::*;

        match self {
            Utf8(_) => "invalid character",
            Int(_) => "cannot parse number",
            Invalid => "invalid value",
            Empty => "nothing to parse",
            StatusErr => "status line contains invalid values",
            HeadersErr => "headers contain invalid values",
            UriErr => "uri contains invalid characters",
        }
    }
}

impl From<num::ParseIntError> for ParseErr {
    fn from(e: num::ParseIntError) -> Self {
        ParseErr::Int(e)
    }
}

impl From<str::Utf8Error> for ParseErr {
    fn from(e: str::Utf8Error) -> Self {
        ParseErr::Utf8(e)
    }
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Parse(ParseErr),
    Tls,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.description())
    }
}

impl StdErr for Error {
    fn description(&self) -> &str {
        use self::Error::*;

        match self {
            IO(_) => "IO error",
            Parse(e) => e.description(),
            Tls => "TLS error",
        }
    }
}

#[cfg(feature = "native-tls")]
impl From<native_tls::Error> for Error {
    fn from(_e: native_tls::Error) -> Self {
        Error::Tls
    }
}

#[cfg(feature = "native-tls")]
impl<T> From<native_tls::HandshakeError<T>> for Error {
    fn from(_e: native_tls::HandshakeError<T>) -> Self {
        Error::Tls
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<ParseErr> for Error {
    fn from(e: ParseErr) -> Self {
        Error::Parse(e)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(e: str::Utf8Error) -> Self {
        Error::Parse(ParseErr::Utf8(e))
    }
}
