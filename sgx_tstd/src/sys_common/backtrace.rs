// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

/// Common code for printing the backtrace in the same way across the different
/// supported platforms.

use sgx_trts::libc;
use io::prelude::*;
use io;
use sync::SgxThreadMutex;
use core::str;
use core::sync::atomic::{self, Ordering};
use core::ptr;

pub use sys::backtrace::{
    unwind_backtrace,
    init_state,
    resolve_symname,
    foreach_symbol_fileline,
    BacktraceContext
};

#[cfg(target_pointer_width = "64")]
pub const HEX_WIDTH: usize = 18;

#[cfg(target_pointer_width = "32")]
pub const HEX_WIDTH: usize = 10;

/// Represents an item in the backtrace list. See `unwind_backtrace` for how
/// it is created.
#[derive(Debug, Copy, Clone)]
pub struct Frame {
    /// Exact address of the call that failed.
    pub exact_position: *const u8,
    /// Address of the enclosing function.
    pub symbol_addr: *const u8,
    /// Which inlined function is this frame referring to
    pub inline_context: u32,
}

/// Max number of frames to print.
const MAX_NB_FRAMES: usize = 100;

/// Prints the current backtrace.
pub fn print(w: &mut Write, format: PrintFormat) -> io::Result<()> {
    static LOCK: SgxThreadMutex = SgxThreadMutex::new();

    // Use a lock to prevent mixed output in multithreading context.
    // Some platforms also requires it, like `SymFromAddr` on Windows.
    unsafe {
        LOCK.lock();
        let res = _print(w, format);
        LOCK.unlock();
        res
    }
}

fn _print(w: &mut Write, format: PrintFormat) -> io::Result<()> {

    let state = init_state();
    if state.is_err() {
        let error = state.as_ref()
                        .unwrap_err()
                        .raw_os_error()
                        .unwrap_or(0);
        if error == libc::ENOENT {
            writeln!(w, "note: Call enclave::set_enclave_path to set the path of enclave file for backtrace.");
        }
        return state;
    }

    let mut frames = [Frame {
        exact_position: ptr::null(),
        symbol_addr: ptr::null(),
        inline_context: 0,
    }; MAX_NB_FRAMES];
    let (nb_frames, context) = unwind_backtrace(&mut frames)?;
    let (skipped_before, skipped_after) =
        filter_frames(&frames[..nb_frames], format, &context);
    if skipped_before + skipped_after > 0 {
        //writeln!(w, "note: Some details are omitted, \
        //             compile with `RUST_BACKTRACE=full` for a verbose backtrace.")?;
        writeln!(w, "note: Some details are omitted, \
                     call backtrace::enable_backtrace with 'PrintFormat::Full' for a verbose backtrace.")?;
    }
    writeln!(w, "stack backtrace:")?;

    let filtered_frames = &frames[..nb_frames - skipped_after];
    for (index, frame) in filtered_frames.iter().skip(skipped_before).enumerate() {
        resolve_symname(*frame, |symname| {
            output(w, index, *frame, symname, format)
        }, &context)?;
        let has_more_filenames = foreach_symbol_fileline(*frame, |file, line| {
            output_fileline(w, file, line, format)
        }, &context)?;
        if has_more_filenames {
            w.write_all(b" <... and possibly more>")?;
        }
    }

    Ok(())
}

/// Returns a number of frames to remove at the beginning and at the end of the
/// backtrace, according to the backtrace format.
fn filter_frames(frames: &[Frame],
                 format: PrintFormat,
                 context: &BacktraceContext) -> (usize, usize)
{
    if format == PrintFormat::Full {
        return (0, 0);
    }

    let skipped_before = 0;

    let skipped_after = frames.len() - frames.iter().position(|frame| {
        let mut is_marker = false;
        let _ = resolve_symname(*frame, |symname| {
            if let Some(mangled_symbol_name) = symname {
                // Use grep to find the concerned functions
                if mangled_symbol_name.contains("__rust_begin_short_backtrace") {
                    is_marker = true;
                }
            }
            Ok(())
        }, context);
        is_marker
    }).unwrap_or(frames.len());

    if skipped_before + skipped_after >= frames.len() {
        // Avoid showing completely empty backtraces
        return (0, 0);
    }

    (skipped_before, skipped_after)
}


/// Fixed frame used to clean the backtrace with `RUST_BACKTRACE=1`.
#[inline(never)]
pub fn __rust_begin_short_backtrace<F, T>(f: F) -> T
    where F: FnOnce() -> T, F: Send, T: Send
{
    f()
}

/// Controls how the backtrace should be formatted.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrintFormat {
    /// Show all the frames with absolute path for files.
    Full = 2,
    /// Show only relevant data from the backtrace.
    Short = 3,
}

static ENABLED: atomic::AtomicIsize = atomic::AtomicIsize::new(1);

// For now logging is turned off by default, and this function checks to see
// whether the magical environment variable is present to see if it's turned on.
pub fn log_enabled() -> Option<PrintFormat> {
    /*
    match ENABLED.load(Ordering::SeqCst) {
        0 => {},
        1 => return None,
        2 => return Some(PrintFormat::Full),
        3 => return Some(PrintFormat::Short),
        _ => unreachable!(),
    }

    let val: Option<PrintFormat>;
    if cfg!(RUST_BACKTRACE = "0") {
        val = None;
    } else if cfg!(RUST_BACKTRACE = "full") {
        val = Some(PrintFormat::Full);
    } else {
        val = Some(PrintFormat::Short);
    }

    ENABLED.store(match val {
        Some(v) => v as isize,
        None => 1,
    }, Ordering::SeqCst);
    val
    */
    match ENABLED.load(Ordering::SeqCst) {
        0 => None,
        1 => None,
        2 => Some(PrintFormat::Full),
        3 => Some(PrintFormat::Short),
        _ => unreachable!(),
    }
}

pub fn set_enabled(backtrace: PrintFormat) {
    ENABLED.store(match backtrace {
        PrintFormat::Full => 2,
        PrintFormat::Short => 3,
    }, Ordering::SeqCst);
}

/// Print the symbol of the backtrace frame.
///
/// These output functions should now be used everywhere to ensure consistency.
/// You may want to also use `output_fileline`.
fn output(w: &mut Write, idx: usize, frame: Frame,
              s: Option<&str>, format: PrintFormat) -> io::Result<()> {
    // Remove the `17: 0x0 - <unknown>` line.
    if format == PrintFormat::Short && frame.exact_position == ptr::null() {
        return Ok(());
    }
    match format {
        PrintFormat::Full => write!(w,
                                    "  {:2}: {:2$?} - ",
                                    idx,
                                    frame.exact_position,
                                    HEX_WIDTH)?,
        PrintFormat::Short => write!(w, "  {:2}: ", idx)?,
    }
    match s {
        Some(string) => demangle(w, &string.to_string(), format)?,
        None => w.write_all(b"<unknown>")?,
    }
    w.write_all(b"\n")
}

/// Print the filename and line number of the backtrace frame.
///
/// See also `output`.
#[allow(dead_code)]
fn output_fileline(w: &mut Write,
                   file: &[u8],
                   line: u32,
                   format: PrintFormat) -> io::Result<()> {
    // prior line: "  ##: {:2$} - func"
    w.write_all(b"")?;
    match format {
        PrintFormat::Full => write!(w,
                                    "           {:1$}",
                                    "",
                                    HEX_WIDTH)?,
        PrintFormat::Short => write!(w, "           ")?,
    }

    /*
    let file = str::from_utf8(file).unwrap_or("<unknown>");
    let file_path = Path::new(file);
    let mut already_printed = false;
    if format == PrintFormat::Short && file_path.is_absolute() {
        if let Ok(cwd) = env::current_dir() {
            if let Ok(stripped) = file_path.strip_prefix(&cwd) {
                if let Some(s) = stripped.to_str() {
                    write!(w, "  at .{}{}:{}", path::MAIN_SEPARATOR, s, line)?;
                    already_printed = true;
                }
            }
        }
    }
    if !already_printed {
        write!(w, "  at {}:{}", file, line)?;
    }
    */

    let file = str::from_utf8(file).unwrap_or("<unknown>").to_string();
    write!(w, "  at {}:{}", &file, line)?;

    w.write_all(b"\n")
}


// All rust symbols are in theory lists of "::"-separated identifiers. Some
// assemblers, however, can't handle these characters in symbol names. To get
// around this, we use C++-style mangling. The mangling method is:
//
// 1. Prefix the symbol with "_ZN"
// 2. For each element of the path, emit the length plus the element
// 3. End the path with "E"
//
// For example, "_ZN4testE" => "test" and "_ZN3foo3barE" => "foo::bar".
//
// We're the ones printing our backtraces, so we can't rely on anything else to
// demangle our symbols. It's *much* nicer to look at demangled symbols, so
// this function is implemented to give us nice pretty output.
//
// Note that this demangler isn't quite as fancy as it could be. We have lots
// of other information in our symbols like hashes, version, type information,
// etc. Additionally, this doesn't handle glue symbols at all.
pub fn demangle(writer: &mut Write, mut s: &str, format: PrintFormat) -> io::Result<()> {
    // During ThinLTO LLVM may import and rename internal symbols, so strip out
    // those endings first as they're one of the last manglings applied to
    // symbol names.
    let llvm = ".llvm.";
    if let Some(i) = s.find(llvm) {
        let candidate = &s[i + llvm.len()..];
        let all_hex = candidate.chars().all(|c| {
            match c {
                'A' ... 'F' | '0' ... '9' => true,
                _ => false,
            }
        });

        if all_hex {
            s = &s[..i];
        }
    }

    // Validate the symbol. If it doesn't look like anything we're
    // expecting, we just print it literally. Note that we must handle non-rust
    // symbols because we could have any function in the backtrace.
    let mut valid = true;
    let mut inner = s;
    if s.len() > 4 && s.starts_with("_ZN") && s.ends_with("E") {
        inner = &s[3 .. s.len() - 1];
    // On Windows, dbghelp strips leading underscores, so we accept "ZN...E" form too.
    } else if s.len() > 3 && s.starts_with("ZN") && s.ends_with("E") {
        inner = &s[2 .. s.len() - 1];
    } else {
        valid = false;
    }

    if valid {
        let mut chars = inner.chars();
        while valid {
            let mut i = 0;
            for c in chars.by_ref() {
                if c.is_numeric() {
                    i = i * 10 + c as usize - '0' as usize;
                } else {
                    break
                }
            }
            if i == 0 {
                valid = chars.next().is_none();
                break
            } else if chars.by_ref().take(i - 1).count() != i - 1 {
                valid = false;
            }
        }
    }

    // Alright, let's do this.
    if !valid {
        writer.write_all(s.as_bytes())?;
    } else {
        // remove the `::hfc2edb670e5eda97` part at the end of the symbol.
        if format == PrintFormat::Short {
            // The symbol in still mangled.
            let mut split = inner.rsplitn(2, "17h");
            match (split.next(), split.next()) {
                (Some(addr), rest) => {
                    if addr.len() == 16 &&
                       addr.chars().all(|c| c.is_digit(16))
                    {
                        inner = rest.unwrap_or("");
                    }
                }
                _ => (),
            }
        }

        let mut first = true;
        while !inner.is_empty() {
            if !first {
                writer.write_all(b"::")?;
            } else {
                first = false;
            }
            let mut rest = inner;
            while rest.chars().next().unwrap().is_numeric() {
                rest = &rest[1..];
            }
            let i: usize = inner[.. (inner.len() - rest.len())].parse().unwrap();
            inner = &rest[i..];
            rest = &rest[..i];
            if rest.starts_with("_$") {
                rest = &rest[1..];
            }
            while !rest.is_empty() {
                if rest.starts_with(".") {
                    if let Some('.') = rest[1..].chars().next() {
                        writer.write_all(b"::")?;
                        rest = &rest[2..];
                    } else {
                        writer.write_all(b".")?;
                        rest = &rest[1..];
                    }
                } else if rest.starts_with("$") {
                    macro_rules! demangle {
                        ($($pat:expr => $demangled:expr),*) => ({
                            $(if rest.starts_with($pat) {
                                writer.write_all($demangled)?;
                                rest = &rest[$pat.len()..];
                              } else)*
                            {
                                writer.write_all(rest.as_bytes())?;
                                break;
                            }

                        })
                    }

                    // see src/librustc/back/link.rs for these mappings
                    demangle! (
                        "$SP$" => b"@",
                        "$BP$" => b"*",
                        "$RF$" => b"&",
                        "$LT$" => b"<",
                        "$GT$" => b">",
                        "$LP$" => b"(",
                        "$RP$" => b")",
                        "$C$" => b",",

                        // in theory we can demangle any Unicode code point, but
                        // for simplicity we just catch the common ones.
                        "$u7e$" => b"~",
                        "$u20$" => b" ",
                        "$u27$" => b"'",
                        "$u5b$" => b"[",
                        "$u5d$" => b"]",
                        "$u7b$" => b"{",
                        "$u7d$" => b"}",
                        "$u3b$" => b";",
                        "$u2b$" => b"+",
                        "$u22$" => b"\""
                    )
                } else {
                    let idx = match rest.char_indices().find(|&(_, c)| c == '$' || c == '.') {
                        None => rest.len(),
                        Some((i, _)) => i,
                    };
                    writer.write_all(rest[..idx].as_bytes())?;
                    rest = &rest[idx..];
                }
            }
        }
    }

    Ok(())
}