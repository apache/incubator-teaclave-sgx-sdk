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
// under the License..

//! Standard library macros
//!
//! This modules contains a set of macros which are exported from the standard
//! library. Each macro is available for use when linking against the standard
//! library.

/// Panics the current thread.
///
/// This allows a program to terminate immediately and provide feedback
/// to the caller of the program. `panic!` should be used when a program reaches
/// an unrecoverable state.
///
/// This macro is the perfect way to assert conditions in example code and in
/// tests. `panic!` is closely tied with the `unwrap` method of both [`Option`]
/// and [`Result`][runwrap] enums. Both implementations call `panic!` when they are set
/// to None or Err variants.
///
/// This macro is used to inject panic into a Rust thread, causing the thread to
/// panic entirely. Each thread's panic can be reaped as the `Box<Any>` type,
/// and the single-argument form of the `panic!` macro will be the value which
/// is transmitted.
///
/// [`Result`] enum is often a better solution for recovering from errors than
/// using the `panic!` macro. This macro should be used to avoid proceeding using
/// incorrect values, such as from external sources. Detailed information about
/// error handling is found in the [book].
///
/// The multi-argument form of this macro panics with a string and has the
/// [`format!`] syntax for building a string.
///
/// # Current implementation
///
/// If the main thread panics it will terminate all your threads and end your
/// program with code `101`.
///
#[macro_export]
#[allow_internal_unstable(libstd_sys_internals)]
macro_rules! panic {
    () => ({ $crate::panic!("explicit panic") });
    ($msg:expr) => ({ $crate::rt::begin_panic($msg) });
    ($msg:expr,) => ({ $crate::panic!($msg) });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::rt::begin_panic_fmt(&$crate::format_args!($fmt, $($arg)+))
    });
}

/// Prints to the standard output.
///
/// Equivalent to the [`println!`] macro except that a newline is not printed at
/// the end of the message.
///
/// Note that stdout is frequently line-buffered by default so it may be
/// necessary to use [`io::stdout().flush()`][flush] to ensure the output is emitted
/// immediately.
///
/// Use `print!` only for the primary output of your program. Use
/// [`eprint!`] instead to print error and progress messages.
///
/// [`println!`]: ../std/macro.println.html
/// [flush]: ../std/io/trait.Write.html#tymethod.flush
/// [`eprint!`]: ../std/macro.eprint.html
///
/// # Panics
///
/// Panics if writing to `io::stdout()` fails.
#[cfg(feature = "stdio")]
#[macro_export]
#[allow_internal_unstable(print_internals)]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print($crate::format_args!($($arg)*)));
}

/// Prints to the standard output, with a newline.
///
/// On all platforms, the newline is the LINE FEED character (`\n`/`U+000A`) alone
/// (no additional CARRIAGE RETURN (`\r`/`U+000D`)).
///
/// Use the [`format!`] syntax to write data to the standard output.
/// See [`std::fmt`] for more information.
///
/// Use `println!` only for the primary output of your program. Use
/// [`eprintln!`] instead to print error and progress messages.
///
/// [`format!`]: ../std/macro.format.html
/// [`std::fmt`]: ../std/fmt/index.html
/// [`eprintln!`]: ../std/macro.eprintln.html
/// # Panics
///
/// Panics if writing to `io::stdout` fails.
#[cfg(feature = "stdio")]
#[macro_export]
#[allow_internal_unstable(print_internals, format_args_nl)]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::io::_print($crate::format_args_nl!($($arg)*));
    })
}

#[cfg(not(feature = "stdio"))]
#[macro_export]
macro_rules! print { ($($arg:tt)*) => ({}) }

#[cfg(not(feature = "stdio"))]
#[macro_export]
macro_rules! println { ($($arg:tt)*) => ({}) }

/// Prints to the standard error.
///
/// Equivalent to the [`print!`] macro, except that output goes to
/// [`io::stderr`] instead of `io::stdout`. See [`print!`] for
/// example usage.
///
/// Use `eprint!` only for error and progress messages. Use `print!`
/// instead for the primary output of your program.
///
/// [`io::stderr`]: ../std/io/struct.Stderr.html
/// [`print!`]: ../std/macro.print.html
///
/// # Panics
///
/// Panics if writing to `io::stderr` fails.
#[cfg(feature = "stdio")]
#[macro_export]
#[allow_internal_unstable(print_internals)]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::io::_eprint($crate::format_args!($($arg)*)));
}

/// Prints to the standard error, with a newline.
///
/// Equivalent to the [`println!`] macro, except that output goes to
/// [`io::stderr`] instead of `io::stdout`. See [`println!`] for
/// example usage.
///
/// Use `eprintln!` only for error and progress messages. Use `println!`
/// instead for the primary output of your program.
///
/// [`io::stderr`]: ../std/io/struct.Stderr.html
/// [`println!`]: ../std/macro.println.html
///
/// # Panics
///
/// Panics if writing to `io::stderr` fails.
#[cfg(feature = "stdio")]
#[macro_export]
#[allow_internal_unstable(print_internals, format_args_nl)]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ({
        $crate::io::_eprint($crate::format_args_nl!($($arg)*));
    })
}

#[cfg(not(feature = "stdio"))]
#[macro_export]
macro_rules! eprint { ($($arg:tt)*) => ({}) }

#[cfg(not(feature = "stdio"))]
#[macro_export]
macro_rules! eprintln { ($($arg:tt)*) => ({}) }


#[macro_export]
macro_rules! dbg {
    () => {
        $crate::eprintln!("[{}:{}]", $crate::file!(), $crate::line!());
    };
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::eprintln!("[{}:{}] {} = {:#?}",
                    $crate::file!(), $crate::line!(), $crate::stringify!($val), &tmp);
                tmp
            }
        }
    };
    // Trailing comma with single argument is ignored
    ($val:expr,) => { $crate::dbg!($val) };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
