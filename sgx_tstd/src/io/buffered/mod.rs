// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Buffering wrappers for I/O traits

mod bufreader;
mod bufwriter;
mod linewriter;
mod linewritershim;

#[cfg(feature = "unit_test")]
mod tests;

use crate::error;
use crate::fmt;
use crate::io::Error;

pub use self::{bufreader::BufReader, bufwriter::BufWriter, linewriter::LineWriter};
use linewritershim::LineWriterShim;

pub use bufwriter::WriterPanicked;

/// An error returned by [`BufWriter::into_inner`] which combines an error that
/// happened while writing out the buffer, and the buffered writer object
/// which may be used to recover from the condition.
///
/// # Examples
///
/// ```no_run
/// use std::io::BufWriter;
/// use std::net::TcpStream;
///
/// let mut stream = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
///
/// // do stuff with the stream
///
/// // we want to get our `TcpStream` back, so let's try:
///
/// let stream = match stream.into_inner() {
///     Ok(s) => s,
///     Err(e) => {
///         // Here, e is an IntoInnerError
///         panic!("An error occurred");
///     }
/// };
/// ```
#[derive(Debug)]
pub struct IntoInnerError<W>(W, Error);

impl<W> IntoInnerError<W> {
    /// Construct a new IntoInnerError
    fn new(writer: W, error: Error) -> Self {
        Self(writer, error)
    }

    /// Helper to construct a new IntoInnerError; intended to help with
    /// adapters that wrap other adapters
    fn new_wrapped<W2>(self, f: impl FnOnce(W) -> W2) -> IntoInnerError<W2> {
        let Self(writer, error) = self;
        IntoInnerError::new(f(writer), error)
    }

    /// Returns the error which caused the call to [`BufWriter::into_inner()`]
    /// to fail.
    ///
    /// This error was returned when attempting to write the internal buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let mut stream = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // do stuff with the stream
    ///
    /// // we want to get our `TcpStream` back, so let's try:
    ///
    /// let stream = match stream.into_inner() {
    ///     Ok(s) => s,
    ///     Err(e) => {
    ///         // Here, e is an IntoInnerError, let's log the inner error.
    ///         //
    ///         // We'll just 'log' to stdout for this example.
    ///         println!("{}", e.error());
    ///
    ///         panic!("An unexpected error occurred.");
    ///     }
    /// };
    /// ```
    pub fn error(&self) -> &Error {
        &self.1
    }

    /// Returns the buffered writer instance which generated the error.
    ///
    /// The returned object can be used for error recovery, such as
    /// re-inspecting the buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let mut stream = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // do stuff with the stream
    ///
    /// // we want to get our `TcpStream` back, so let's try:
    ///
    /// let stream = match stream.into_inner() {
    ///     Ok(s) => s,
    ///     Err(e) => {
    ///         // Here, e is an IntoInnerError, let's re-examine the buffer:
    ///         let buffer = e.into_inner();
    ///
    ///         // do stuff to try to recover
    ///
    ///         // afterwards, let's just return the stream
    ///         buffer.into_inner().unwrap()
    ///     }
    /// };
    /// ```
    pub fn into_inner(self) -> W {
        self.0
    }

    /// Consumes the [`IntoInnerError`] and returns the error which caused the call to
    /// [`BufWriter::into_inner()`] to fail.  Unlike `error`, this can be used to
    /// obtain ownership of the underlying error.
    ///
    /// # Example
    /// ```
    /// use std::io::{BufWriter, ErrorKind, Write};
    ///
    /// let mut not_enough_space = [0u8; 10];
    /// let mut stream = BufWriter::new(not_enough_space.as_mut());
    /// write!(stream, "this cannot be actually written").unwrap();
    /// let into_inner_err = stream.into_inner().expect_err("now we discover it's too small");
    /// let err = into_inner_err.into_error();
    /// assert_eq!(err.kind(), ErrorKind::WriteZero);
    /// ```
    pub fn into_error(self) -> Error {
        self.1
    }

    /// Consumes the [`IntoInnerError`] and returns the error which caused the call to
    /// [`BufWriter::into_inner()`] to fail, and the underlying writer.
    ///
    /// This can be used to simply obtain ownership of the underlying error; it can also be used for
    /// advanced error recovery.
    ///
    /// # Example
    /// ```
    /// use std::io::{BufWriter, ErrorKind, Write};
    ///
    /// let mut not_enough_space = [0u8; 10];
    /// let mut stream = BufWriter::new(not_enough_space.as_mut());
    /// write!(stream, "this cannot be actually written").unwrap();
    /// let into_inner_err = stream.into_inner().expect_err("now we discover it's too small");
    /// let (err, recovered_writer) = into_inner_err.into_parts();
    /// assert_eq!(err.kind(), ErrorKind::WriteZero);
    /// assert_eq!(recovered_writer.buffer(), b"t be actually written");
    /// ```
    pub fn into_parts(self) -> (Error, W) {
        (self.1, self.0)
    }
}

impl<W> From<IntoInnerError<W>> for Error {
    fn from(iie: IntoInnerError<W>) -> Error {
        iie.1
    }
}

impl<W: Send + fmt::Debug> error::Error for IntoInnerError<W> {
    #[allow(deprecated, deprecated_in_future)]
    fn description(&self) -> &str {
        error::Error::description(self.error())
    }
}

impl<W> fmt::Display for IntoInnerError<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error().fmt(f)
    }
}
