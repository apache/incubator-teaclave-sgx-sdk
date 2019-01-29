use std::error;
use std::fmt;
use std::io;

/// Create a new error from a kind without a line number.
pub fn error_new(kind: ErrorKind) -> Error {
    Error { kind: kind, line: None }
}

/// Create a new parse error from the given message.
pub fn error_parse(msg: String) -> Error {
    error_new(ErrorKind::Parse(msg))
}

/// Set the line number on the given error.
pub fn error_set_line(err: &mut Error, line: Option<u64>) {
    err.line = line;
}

/// Represents any kind of error that can occur while parsing the UCD.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    line: Option<u64>,
}

/// The kind of error that occurred while parsing the UCD.
#[derive(Debug)]
pub enum ErrorKind {
    /// An I/O error.
    Io(io::Error),
    /// A generic parse error.
    Parse(String),
}

impl Error {
    /// Return the specific kind of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Return the line number at which this error occurred, if available.
    pub fn line(&self) -> Option<u64> {
        self.line
    }

    /// Unwrap this error into its underlying kind.
    pub fn into_kind(self) -> ErrorKind {
        self.kind
    }

    /// Returns true if and only if this is an I/O error.
    ///
    /// If this returns true, the underlying `ErrorKind` is guaranteed to be
    /// `ErrorKind::Io`.
    pub fn is_io_error(&self) -> bool {
        match self.kind {
            ErrorKind::Io(_) => true,
            _ => false,
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Io(ref err) => err.description(),
            ErrorKind::Parse(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.kind {
            ErrorKind::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Io(ref err) => err.fmt(f),
            ErrorKind::Parse(ref msg) => {
                if let Some(line) = self.line {
                    write!(f, "error on line {}: {}", line, msg)
                } else {
                    write!(f, "{}", msg)
                }
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error { kind: ErrorKind::Io(err), line: None }
    }
}
