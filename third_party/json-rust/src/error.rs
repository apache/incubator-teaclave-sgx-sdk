use std::{ char, error, fmt };
use std::string::String;
/// Error type of this crate.
///
///
/// *Note:* Since `0.9.0` using `JsonError` is deprecated. Always use
/// `json::Error` instead!
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UnexpectedCharacter {
        ch: char,
        line: usize,
        column: usize,
    },
    UnexpectedEndOfJson,
    ExceededDepthLimit,
    FailedUtf8Parsing,
    WrongType(String),
}

impl Error {
    pub fn wrong_type(expected: &str) -> Self {
        Error::WrongType(expected.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match *self {
            UnexpectedCharacter {
                ref ch,
                ref line,
                ref column,
            } => write!(f, "Unexpected character: {} at ({}:{})", ch, line, column),

            UnexpectedEndOfJson   => write!(f, "Unexpected end of JSON"),
            ExceededDepthLimit    => write!(f, "Exceeded depth limit"),
            FailedUtf8Parsing     => write!(f, "Failed to parse UTF-8 bytes"),
            WrongType(ref s)      => write!(f, "Wrong type, expected: {}", s),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use Error::*;

        match *self {
            UnexpectedCharacter { .. } => "Unexpected character",
            UnexpectedEndOfJson        => "Unexpected end of JSON",
            ExceededDepthLimit         => "Exceeded depth limit",
            FailedUtf8Parsing          => "Failed to read bytes as UTF-8 from JSON",
            WrongType(_)               => "Wrong type",
        }
    }
}
