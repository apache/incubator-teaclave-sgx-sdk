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

pub use self::printing::{BacktraceFmt, BacktraceFrameFmt, PrintFmt};
/// Backtrace support built on libgcc with some extra OS-specific support
///
/// Some methods of getting a backtrace:
///
/// * The backtrace() functions on unix. It turns out this doesn't work very
///   well for green threads on macOS, and the address to symbol portion of it
///   suffers problems that are described below.
///
/// * Using libunwind. This is more difficult than it sounds because libunwind
///   isn't installed everywhere by default. It's also a bit of a hefty library,
///   so possibly not the best option. When testing, libunwind was excellent at
///   getting both accurate backtraces and accurate symbols across platforms.
///   This route was not chosen in favor of the next option, however.
///
/// * We're already using libgcc_s for exceptions in rust (triggering thread
///   unwinding and running destructors on the stack), and it turns out that it
///   conveniently comes with a function that also gives us a backtrace. All of
///   these functions look like _Unwind_*, but it's not quite the full
///   repertoire of the libunwind API. Due to it already being in use, this was
///   the chosen route of getting a backtrace.
///
/// After choosing libgcc_s for backtraces, the sad part is that it will only
/// give us a stack trace of instruction pointers. Thankfully these instruction
/// pointers are accurate (they work for green and native threads), but it's
/// then up to us again to figure out how to translate these addresses to
/// symbols. As with before, we have a few options. Before, that, a little bit
/// of an interlude about symbols. This is my very limited knowledge about
/// symbol tables, and this information is likely slightly wrong, but the
/// general idea should be correct.
///
/// When talking about symbols, it's helpful to know a few things about where
/// symbols are located. Some symbols are located in the dynamic symbol table
/// of the executable which in theory means that they're available for dynamic
/// linking and lookup. Other symbols end up only in the local symbol table of
/// the file. This loosely corresponds to pub and priv functions in Rust.
///
/// Armed with this knowledge, we know that our solution for address to symbol
/// translation will need to consult both the local and dynamic symbol tables.
/// With that in mind, here's our options of translating an address to
/// a symbol.
///
/// * Use dladdr(). The original backtrace()-based idea actually uses dladdr()
///   behind the scenes to translate, and this is why backtrace() was not used.
///   Conveniently, this method works fantastically on macOS. It appears dladdr()
///   uses magic to consult the local symbol table, or we're putting everything
///   in the dynamic symbol table anyway. Regardless, for macOS, this is the
///   method used for translation. It's provided by the system and easy to do.o
///
///   Sadly, all other systems have a dladdr() implementation that does not
///   consult the local symbol table. This means that most functions are blank
///   because they don't have symbols. This means that we need another solution.
///
/// * Use unw_get_proc_name(). This is part of the libunwind api (not the
///   libgcc_s version of the libunwind api), but involves taking a dependency
///   to libunwind. We may pursue this route in the future if we bundle
///   libunwind, but libunwind was unwieldy enough that it was not chosen at
///   this time to provide this functionality.
///
/// * Shell out to a utility like `readelf`. Crazy though it may sound, it's a
///   semi-reasonable solution. The stdlib already knows how to spawn processes,
///   so in theory it could invoke readelf, parse the output, and consult the
///   local/dynamic symbol tables from there. This ended up not getting chosen
///   due to the craziness of the idea plus the advent of the next option.
///
/// * Use `libbacktrace`. It turns out that this is a small library bundled in
///   the gcc repository which provides backtrace and symbol translation
///   functionality. All we really need from it is the backtrace functionality,
///   and we only really need this on everything that's not macOS, so this is the
///   chosen route for now.
///
/// In summary, the current situation uses libgcc_s to get a trace of stack
/// pointers, and we use dladdr() or libbacktrace to translate these addresses
/// to symbols. This is a bit of a hokey implementation as-is, but it works for
/// all unix platforms we support right now, so it at least gets the job done.
pub use self::tracing::{trace_unsynchronized, Frame};

// tracing impls:
mod tracing;
// symbol resolvers:
mod printing;

/// A platform independent representation of a string. When working with `std`
/// enabled it is recommended to the convenience methods for providing
/// conversions to `std` types.
#[derive(Debug)]
pub enum BytesOrWideString<'a> {
    /// A slice, typically provided on Unix platforms.
    Bytes(&'a [u8]),
    /// Wide strings typically from Windows.
    Wide(&'a [u16]),
}

pub struct Bomb {
    enabled: bool,
}

impl Bomb {
    pub fn new(enabled: bool) -> Bomb {
        Bomb { enabled }
    }

    pub fn set(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Drop for Bomb {
    fn drop(&mut self) {
        assert!(!self.enabled, "cannot panic during the backtrace function");
    }
}

pub mod gnu {
    use crate::enclave;
    use crate::ffi::CString;
    use crate::io::{self, Error, ErrorKind};
    use crate::os::unix::ffi::OsStrExt;

    pub fn get_enclave_filename() -> io::Result<Vec<u8>> {
        let p = enclave::get_enclave_path();
        let result = match p {
            None => Err(Error::new(ErrorKind::Other, "no enclave path found")),
            Some(path) => {
                let cstr = CString::new(path.as_os_str().as_bytes())?;
                let v = cstr.into_bytes_with_nul();
                Ok(v)
            }
        };
        result
    }
}

pub struct BacktraceContext;