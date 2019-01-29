// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! Standard library macros
//!
//! This modules contains a set of macros which are exported from the standard
//! library. Each macro is available for use when linking against the standard
//! library.

/// The entry point for panic of Rust threads.
///
/// This allows a program to to terminate immediately and provide feedback
/// to the caller of the program. `panic!` should be used when a program reaches
/// an unrecoverable problem.
///
/// This macro is the perfect way to assert conditions in example code and in
/// tests.  `panic!` is closely tied with the `unwrap` method of both [`Option`]
/// and [`Result`][runwrap] enums.  Both implementations call `panic!` when they are set
/// to None or Err variants.
///
/// This macro is used to inject panic into a Rust thread, causing the thread to
/// panic entirely. Each thread's panic can be reaped as the `Box<Any>` type,
/// and the single-argument form of the `panic!` macro will be the value which
/// is transmitted.
///
/// [`Result`] enum is often a better solution for recovering from errors than
/// using the `panic!` macro.  This macro should be used to avoid proceeding using
/// incorrect values, such as from external sources.  Detailed information about
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
#[allow_internal_unstable]
macro_rules! panic {
    () => ({
        panic!("explicit panic")
    });
    ($msg:expr) => ({
        $crate::rt::begin_panic($msg, &(file!(), line!(), column!()))
    });
    ($msg:expr,) => ({
        panic!($msg)
    });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::rt::begin_panic_fmt(&format_args!($fmt, $($arg)+),
                                    &(file!(), line!(), column!()))
    });
}

/// Macro for printing to the standard output.
///
/// Equivalent to the `println!` macro except that a newline is not printed at
/// the end of the message.
///
/// Note that stdout is frequently line-buffered by default so it may be
/// necessary to use `io::stdout().flush()` to ensure the output is emitted
/// immediately.
///
/// Use `print!` only for the primary output of your program.  Use
/// `eprint!` instead to print error and progress messages.
///
/// # Panics
///
/// Panics if writing to `io::stdout()` fails.
#[cfg(feature = "stdio")]
#[macro_export]
#[allow_internal_unstable]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}

/// Macro for printing to the standard output, with a newline.
/// On all platforms, the newline is the LINE FEED character (`\n`/`U+000A`) alone
/// (no additional CARRIAGE RETURN (`\r`/`U+000D`).
///
/// Use the `format!` syntax to write data to the standard output.
/// See `std::fmt` for more information.
///
/// Use `println!` only for the primary output of your program.  Use
/// `eprintln!` instead to print error and progress messages.
///
/// # Panics
///
/// Panics if writing to `io::stdout` fails.
#[cfg(feature = "stdio")]
#[macro_export]
#[allow_internal_unstable]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        $crate::io::_print(format_args_nl!($($arg)*));
    })
}

#[cfg(not(feature = "stdio"))]
#[macro_export]
macro_rules! print { ($($arg:tt)*) => ({}) }

#[cfg(not(feature = "stdio"))]
#[macro_export]
macro_rules! println { ($($arg:tt)*) => ({}) }

/// Macro for printing to the standard error.
///
/// Equivalent to the `print!` macro, except that output goes to
/// `io::stderr` instead of `io::stdout`.  See `print!` for
/// example usage.
///
/// Use `eprint!` only for error and progress messages.  Use `print!`
/// instead for the primary output of your program.
///
/// # Panics
///
/// Panics if writing to `io::stderr` fails.
#[cfg(feature = "stdio")]
#[macro_export]
#[allow_internal_unstable]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::io::_eprint(format_args!($($arg)*)));
}

/// Macro for printing to the standard error, with a newline.
///
/// Equivalent to the `println!` macro, except that output goes to
/// `io::stderr` instead of `io::stdout`.  See `println!` for
/// example usage.
///
/// Use `eprintln!` only for error and progress messages.  Use `println!`
/// instead for the primary output of your program.
///
/// # Panics
///
/// Panics if writing to `io::stderr` fails.
#[cfg(feature = "stdio")]
#[macro_export]
#[allow_internal_unstable]
macro_rules! eprintln {
    () => (eprint!("\n"));
    ($($arg:tt)*) => ({
        $crate::io::_eprint(format_args_nl!($($arg)*));
    })
}

#[cfg(not(feature = "stdio"))]
#[macro_export]
macro_rules! eprint { ($($arg:tt)*) => ({}) }

#[cfg(not(feature = "stdio"))]
#[macro_export]
macro_rules! eprintln { ($($arg:tt)*) => ({}) }

/// A macro for quick and dirty debugging with which you can inspect
/// the value of a given expression. An example:
///
/// ```rust
/// #![feature(dbg_macro)]
///
/// let a = 2;
/// let b = dbg!(a * 2) + 1;
/// //      ^-- prints: [src/main.rs:4] a * 2 = 4
/// assert_eq!(b, 5);
/// ```
///
/// The macro works by using the `Debug` implementation of the type of
/// the given expression to print the value to [stderr] along with the
/// source location of the macro invocation as well as the source code
/// of the expression.
///
/// Invoking the macro on an expression moves and takes ownership of it
/// before returning the evaluated expression unchanged. If the type
/// of the expression does not implement `Copy` and you don't want
/// to give up ownership, you can instead borrow with `dbg!(&expr)`
/// for some expression `expr`.
///
/// Note that the macro is intended as a debugging tool and therefore you
/// should avoid having uses of it in version control for longer periods.
/// Use cases involving debug output that should be added to version control
/// may be better served by macros such as `debug!` from the `log` crate.
///
/// # Stability
///
/// The exact output printed by this macro should not be relied upon
/// and is subject to future changes.
///
/// # Panics
///
/// Panics if writing to `io::stderr` fails.
///
/// # Further examples
///
/// With a method call:
///
/// ```rust
/// #![feature(dbg_macro)]
///
/// fn foo(n: usize) {
///     if let Some(_) = dbg!(n.checked_sub(4)) {
///         // ...
///     }
/// }
///
/// foo(3)
/// ```
///
/// This prints to [stderr]:
///
/// ```text,ignore
/// [src/main.rs:4] n.checked_sub(4) = None
/// ```
///
/// Naive factorial implementation:
///
/// ```rust
/// #![feature(dbg_macro)]
///
/// fn factorial(n: u32) -> u32 {
///     if dbg!(n <= 1) {
///         dbg!(1)
///     } else {
///         dbg!(n * factorial(n - 1))
///     }
/// }
///
/// dbg!(factorial(4));
/// ```
///
/// This prints to [stderr]:
///
/// ```text,ignore
/// [src/main.rs:3] n <= 1 = false
/// [src/main.rs:3] n <= 1 = false
/// [src/main.rs:3] n <= 1 = false
/// [src/main.rs:3] n <= 1 = true
/// [src/main.rs:4] 1 = 1
/// [src/main.rs:5] n * factorial(n - 1) = 2
/// [src/main.rs:5] n * factorial(n - 1) = 6
/// [src/main.rs:5] n * factorial(n - 1) = 24
/// [src/main.rs:11] factorial(4) = 24
/// ```
///
/// The `dbg!(..)` macro moves the input:
///
/// ```compile_fail
/// #![feature(dbg_macro)]
///
/// /// A wrapper around `usize` which importantly is not Copyable.
/// #[derive(Debug)]
/// struct NoCopy(usize);
///
/// let a = NoCopy(42);
/// let _ = dbg!(a); // <-- `a` is moved here.
/// let _ = dbg!(a); // <-- `a` is moved again; error!
/// ```
///
/// [stderr]: https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr)
#[macro_export]
macro_rules! dbg {
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                eprintln!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    }
}

#[macro_export]
#[allow_internal_unstable]
#[allow_internal_unsafe]
macro_rules! await {
    ($e:expr) => { {
        let mut pinned = $e;
        loop {
            if let $crate::task::Poll::Ready(x) =
                $crate::future::poll_with_tls_waker(unsafe {
                    $crate::pin::Pin::new_unchecked(&mut pinned)
                })
            {
                break x;
            }
            // FIXME(cramertj) prior to stabilizing await, we have to ensure that this
            // can't be used to create a generator on stable via `|| await!()`.
            yield
        }
    } }
}
