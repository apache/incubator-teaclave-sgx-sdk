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

//! Traits for working with Errors.

#[cfg(feature = "unit_test")]
mod tests;

#[cfg(feature = "backtrace")]
use crate::backtrace::Backtrace;
use crate::fmt::{self, Write};

pub use core::error::Error;
pub use core::error::{request_ref, request_value, Request};

mod private {
    // This is a hack to prevent `type_id` from being overridden by `Error`
    // implementations, since that can enable unsound downcasting.
    #[derive(Debug)]
    pub struct Internal;
}

/// An error reporter that prints an error and its sources.
///
/// Report also exposes configuration options for formatting the error sources, either entirely on a
/// single line, or in multi-line format with each source on a new line.
///
/// `Report` only requires that the wrapped error implement `Error`. It doesn't require that the
/// wrapped error be `Send`, `Sync`, or `'static`.
///
/// # Examples
///
/// ```rust
/// #![feature(error_reporter)]
/// use std::error::{Error, Report};
/// use std::fmt;
///
/// #[derive(Debug)]
/// struct SuperError {
///     source: SuperErrorSideKick,
/// }
///
/// impl fmt::Display for SuperError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "SuperError is here!")
///     }
/// }
///
/// impl Error for SuperError {
///     fn source(&self) -> Option<&(dyn Error + 'static)> {
///         Some(&self.source)
///     }
/// }
///
/// #[derive(Debug)]
/// struct SuperErrorSideKick;
///
/// impl fmt::Display for SuperErrorSideKick {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "SuperErrorSideKick is here!")
///     }
/// }
///
/// impl Error for SuperErrorSideKick {}
///
/// fn get_super_error() -> Result<(), SuperError> {
///     Err(SuperError { source: SuperErrorSideKick })
/// }
///
/// fn main() {
///     match get_super_error() {
///         Err(e) => println!("Error: {}", Report::new(e)),
///         _ => println!("No error"),
///     }
/// }
/// ```
///
/// This example produces the following output:
///
/// ```console
/// Error: SuperError is here!: SuperErrorSideKick is here!
/// ```
///
/// ## Output consistency
///
/// Report prints the same output via `Display` and `Debug`, so it works well with
/// [`Result::unwrap`]/[`Result::expect`] which print their `Err` variant via `Debug`:
///
/// ```should_panic
/// #![feature(error_reporter)]
/// use std::error::Report;
/// # use std::error::Error;
/// # use std::fmt;
/// # #[derive(Debug)]
/// # struct SuperError {
/// #     source: SuperErrorSideKick,
/// # }
/// # impl fmt::Display for SuperError {
/// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
/// #         write!(f, "SuperError is here!")
/// #     }
/// # }
/// # impl Error for SuperError {
/// #     fn source(&self) -> Option<&(dyn Error + 'static)> {
/// #         Some(&self.source)
/// #     }
/// # }
/// # #[derive(Debug)]
/// # struct SuperErrorSideKick;
/// # impl fmt::Display for SuperErrorSideKick {
/// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
/// #         write!(f, "SuperErrorSideKick is here!")
/// #     }
/// # }
/// # impl Error for SuperErrorSideKick {}
/// # fn get_super_error() -> Result<(), SuperError> {
/// #     Err(SuperError { source: SuperErrorSideKick })
/// # }
///
/// get_super_error().map_err(Report::new).unwrap();
/// ```
///
/// This example produces the following output:
///
/// ```console
/// thread 'main' panicked at src/error.rs:34:40:
/// called `Result::unwrap()` on an `Err` value: SuperError is here!: SuperErrorSideKick is here!
/// note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
/// ```
///
/// ## Return from `main`
///
/// `Report` also implements `From` for all types that implement [`Error`]; this when combined with
/// the `Debug` output means `Report` is an ideal starting place for formatting errors returned
/// from `main`.
///
/// ```should_panic
/// #![feature(error_reporter)]
/// use std::error::Report;
/// # use std::error::Error;
/// # use std::fmt;
/// # #[derive(Debug)]
/// # struct SuperError {
/// #     source: SuperErrorSideKick,
/// # }
/// # impl fmt::Display for SuperError {
/// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
/// #         write!(f, "SuperError is here!")
/// #     }
/// # }
/// # impl Error for SuperError {
/// #     fn source(&self) -> Option<&(dyn Error + 'static)> {
/// #         Some(&self.source)
/// #     }
/// # }
/// # #[derive(Debug)]
/// # struct SuperErrorSideKick;
/// # impl fmt::Display for SuperErrorSideKick {
/// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
/// #         write!(f, "SuperErrorSideKick is here!")
/// #     }
/// # }
/// # impl Error for SuperErrorSideKick {}
/// # fn get_super_error() -> Result<(), SuperError> {
/// #     Err(SuperError { source: SuperErrorSideKick })
/// # }
///
/// fn main() -> Result<(), Report<SuperError>> {
///     get_super_error()?;
///     Ok(())
/// }
/// ```
///
/// This example produces the following output:
///
/// ```console
/// Error: SuperError is here!: SuperErrorSideKick is here!
/// ```
///
/// **Note**: `Report`s constructed via `?` and `From` will be configured to use the single line
/// output format. If you want to make sure your `Report`s are pretty printed and include backtrace
/// you will need to manually convert and enable those flags.
///
/// ```should_panic
/// #![feature(error_reporter)]
/// use std::error::Report;
/// # use std::error::Error;
/// # use std::fmt;
/// # #[derive(Debug)]
/// # struct SuperError {
/// #     source: SuperErrorSideKick,
/// # }
/// # impl fmt::Display for SuperError {
/// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
/// #         write!(f, "SuperError is here!")
/// #     }
/// # }
/// # impl Error for SuperError {
/// #     fn source(&self) -> Option<&(dyn Error + 'static)> {
/// #         Some(&self.source)
/// #     }
/// # }
/// # #[derive(Debug)]
/// # struct SuperErrorSideKick;
/// # impl fmt::Display for SuperErrorSideKick {
/// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
/// #         write!(f, "SuperErrorSideKick is here!")
/// #     }
/// # }
/// # impl Error for SuperErrorSideKick {}
/// # fn get_super_error() -> Result<(), SuperError> {
/// #     Err(SuperError { source: SuperErrorSideKick })
/// # }
///
/// fn main() -> Result<(), Report<SuperError>> {
///     get_super_error()
///         .map_err(Report::from)
///         .map_err(|r| r.pretty(true).show_backtrace(true))?;
///     Ok(())
/// }
/// ```
///
/// This example produces the following output:
///
/// ```console
/// Error: SuperError is here!
///
/// Caused by:
///       SuperErrorSideKick is here!
/// ```
pub struct Report<E = Box<dyn Error>> {
    /// The error being reported.
    error: E,
    /// Whether a backtrace should be included as part of the report.
    show_backtrace: bool,
    /// Whether the report should be pretty-printed.
    pretty: bool,
}

impl<E> Report<E>
where
    Report<E>: From<E>,
{
    /// Create a new `Report` from an input error.
    pub fn new(error: E) -> Report<E> {
        Self::from(error)
    }
}

impl<E> Report<E> {
    /// Enable pretty-printing the report across multiple lines.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #![feature(error_reporter)]
    /// use std::error::Report;
    /// # use std::error::Error;
    /// # use std::fmt;
    /// # #[derive(Debug)]
    /// # struct SuperError {
    /// #     source: SuperErrorSideKick,
    /// # }
    /// # impl fmt::Display for SuperError {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "SuperError is here!")
    /// #     }
    /// # }
    /// # impl Error for SuperError {
    /// #     fn source(&self) -> Option<&(dyn Error + 'static)> {
    /// #         Some(&self.source)
    /// #     }
    /// # }
    /// # #[derive(Debug)]
    /// # struct SuperErrorSideKick;
    /// # impl fmt::Display for SuperErrorSideKick {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "SuperErrorSideKick is here!")
    /// #     }
    /// # }
    /// # impl Error for SuperErrorSideKick {}
    ///
    /// let error = SuperError { source: SuperErrorSideKick };
    /// let report = Report::new(error).pretty(true);
    /// eprintln!("Error: {report:?}");
    /// ```
    ///
    /// This example produces the following output:
    ///
    /// ```console
    /// Error: SuperError is here!
    ///
    /// Caused by:
    ///       SuperErrorSideKick is here!
    /// ```
    ///
    /// When there are multiple source errors the causes will be numbered in order of iteration
    /// starting from the outermost error.
    ///
    /// ```rust
    /// #![feature(error_reporter)]
    /// use std::error::Report;
    /// # use std::error::Error;
    /// # use std::fmt;
    /// # #[derive(Debug)]
    /// # struct SuperError {
    /// #     source: SuperErrorSideKick,
    /// # }
    /// # impl fmt::Display for SuperError {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "SuperError is here!")
    /// #     }
    /// # }
    /// # impl Error for SuperError {
    /// #     fn source(&self) -> Option<&(dyn Error + 'static)> {
    /// #         Some(&self.source)
    /// #     }
    /// # }
    /// # #[derive(Debug)]
    /// # struct SuperErrorSideKick {
    /// #     source: SuperErrorSideKickSideKick,
    /// # }
    /// # impl fmt::Display for SuperErrorSideKick {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "SuperErrorSideKick is here!")
    /// #     }
    /// # }
    /// # impl Error for SuperErrorSideKick {
    /// #     fn source(&self) -> Option<&(dyn Error + 'static)> {
    /// #         Some(&self.source)
    /// #     }
    /// # }
    /// # #[derive(Debug)]
    /// # struct SuperErrorSideKickSideKick;
    /// # impl fmt::Display for SuperErrorSideKickSideKick {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "SuperErrorSideKickSideKick is here!")
    /// #     }
    /// # }
    /// # impl Error for SuperErrorSideKickSideKick { }
    ///
    /// let source = SuperErrorSideKickSideKick;
    /// let source = SuperErrorSideKick { source };
    /// let error = SuperError { source };
    /// let report = Report::new(error).pretty(true);
    /// eprintln!("Error: {report:?}");
    /// ```
    ///
    /// This example produces the following output:
    ///
    /// ```console
    /// Error: SuperError is here!
    ///
    /// Caused by:
    ///    0: SuperErrorSideKick is here!
    ///    1: SuperErrorSideKickSideKick is here!
    /// ```
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Display backtrace if available when using pretty output format.
    ///
    /// # Examples
    ///
    /// **Note**: Report will search for the first `Backtrace` it can find starting from the
    /// outermost error. In this example it will display the backtrace from the second error in the
    /// sources, `SuperErrorSideKick`.
    ///
    /// ```rust
    /// #![feature(error_reporter)]
    /// #![feature(error_generic_member_access)]
    /// # use std::error::Error;
    /// # use std::fmt;
    /// use std::error::Request;
    /// use std::error::Report;
    /// use std::backtrace::Backtrace;
    ///
    /// # #[derive(Debug)]
    /// # struct SuperError {
    /// #     source: SuperErrorSideKick,
    /// # }
    /// # impl fmt::Display for SuperError {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "SuperError is here!")
    /// #     }
    /// # }
    /// # impl Error for SuperError {
    /// #     fn source(&self) -> Option<&(dyn Error + 'static)> {
    /// #         Some(&self.source)
    /// #     }
    /// # }
    /// #[derive(Debug)]
    /// struct SuperErrorSideKick {
    ///     backtrace: Backtrace,
    /// }
    ///
    /// impl SuperErrorSideKick {
    ///     fn new() -> SuperErrorSideKick {
    ///         SuperErrorSideKick { backtrace: Backtrace::force_capture() }
    ///     }
    /// }
    ///
    /// impl Error for SuperErrorSideKick {
    ///     fn provide<'a>(&'a self, request: &mut Request<'a>) {
    ///         request.provide_ref::<Backtrace>(&self.backtrace);
    ///     }
    /// }
    ///
    /// // The rest of the example is unchanged ...
    /// # impl fmt::Display for SuperErrorSideKick {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "SuperErrorSideKick is here!")
    /// #     }
    /// # }
    ///
    /// let source = SuperErrorSideKick::new();
    /// let error = SuperError { source };
    /// let report = Report::new(error).pretty(true).show_backtrace(true);
    /// eprintln!("Error: {report:?}");
    /// ```
    ///
    /// This example produces something similar to the following output:
    ///
    /// ```console
    /// Error: SuperError is here!
    ///
    /// Caused by:
    ///       SuperErrorSideKick is here!
    ///
    /// Stack backtrace:
    ///    0: rust_out::main::_doctest_main_src_error_rs_1158_0::SuperErrorSideKick::new
    ///    1: rust_out::main::_doctest_main_src_error_rs_1158_0
    ///    2: rust_out::main
    ///    3: core::ops::function::FnOnce::call_once
    ///    4: std::sys_common::backtrace::__rust_begin_short_backtrace
    ///    5: std::rt::lang_start::{{closure}}
    ///    6: std::panicking::try
    ///    7: std::rt::lang_start_internal
    ///    8: std::rt::lang_start
    ///    9: main
    ///   10: __libc_start_main
    ///   11: _start
    /// ```
    pub fn show_backtrace(mut self, show_backtrace: bool) -> Self {
        self.show_backtrace = show_backtrace;
        self
    }
}

impl<E> Report<E>
where
    E: Error,
{
    #[cfg(feature = "backtrace")]
    #[allow(clippy::map_flatten)]
    fn backtrace(&self) -> Option<&Backtrace> {
        // have to grab the backtrace on the first error directly since that error may not be
        // 'static
        let backtrace = request_ref(&self.error);
        let backtrace = backtrace.or_else(|| {
            self.error
                .source()
                .map(|source| source.sources().find_map(|source| request_ref(source)))
                .flatten()
        });
        backtrace
    }

    /// Format the report as a single line.
    fn fmt_singleline(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;

        let sources = self.error.source().into_iter().flat_map(<dyn Error>::sources);

        for cause in sources {
            write!(f, ": {cause}")?;
        }

        Ok(())
    }

    /// Format the report as multiple lines, with each error cause on its own line.
    fn fmt_multiline(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = &self.error;

        write!(f, "{error}")?;

        if let Some(cause) = error.source() {
            write!(f, "\n\nCaused by:")?;

            let multiple = cause.source().is_some();

            for (ind, error) in cause.sources().enumerate() {
                writeln!(f)?;
                let mut indented = Indented { inner: f };
                if multiple {
                    write!(indented, "{ind: >4}: {error}")?;
                } else {
                    write!(indented, "      {error}")?;
                }
            }
        }

        #[cfg(feature = "backtrace")]
        if self.show_backtrace {
            let backtrace = self.backtrace();

            if let Some(backtrace) = backtrace {
                let backtrace = backtrace.to_string();

                f.write_str("\n\nStack backtrace:\n")?;
                f.write_str(backtrace.trim_end())?;
            }
        }

        Ok(())
    }
}

impl<E> From<E> for Report<E>
where
    E: Error,
{
    fn from(error: E) -> Self {
        Report { error, show_backtrace: false, pretty: false }
    }
}

impl<E> fmt::Display for Report<E>
where
    E: Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.pretty { self.fmt_multiline(f) } else { self.fmt_singleline(f) }
    }
}

// This type intentionally outputs the same format for `Display` and `Debug`for
// situations where you unwrap a `Report` or return it from main.
impl<E> fmt::Debug for Report<E>
where
    Report<E>: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Wrapper type for indenting the inner source.
struct Indented<'a, D> {
    inner: &'a mut D,
}

impl<T> Write for Indented<'_, T>
where
    T: Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (i, line) in s.split('\n').enumerate() {
            if i > 0 {
                self.inner.write_char('\n')?;
                self.inner.write_str("      ")?;
            }

            self.inner.write_str(line)?;
        }

        Ok(())
    }
}
