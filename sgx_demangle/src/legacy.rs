use core::fmt;

/// Representation of a demangled symbol name.
pub struct Demangle<'a> {
    inner: &'a str,
    /// The number of ::-separated elements in the original name.
    elements: usize,
}

/// De-mangles a Rust symbol into a more readable version
///
/// All Rust symbols by default are mangled as they contain characters that
/// cannot be represented in all object files. The mangling mechanism is similar
/// to C++'s, but Rust has a few specifics to handle items like lifetimes in
/// symbols.
///
/// This function will take a **mangled** symbol and return a value. When printed,
/// the de-mangled version will be written. If the symbol does not look like
/// a mangled symbol, the original value will be written instead.
///
/// # Examples
///
/// ```
/// use rustc_demangle::demangle;
///
/// assert_eq!(demangle("_ZN4testE").to_string(), "test");
/// assert_eq!(demangle("_ZN3foo3barE").to_string(), "foo::bar");
/// assert_eq!(demangle("foo").to_string(), "foo");
/// ```

// All Rust symbols are in theory lists of "::"-separated identifiers. Some
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
pub fn demangle(s: &str) -> Result<Demangle, ()> {
    // First validate the symbol. If it doesn't look like anything we're
    // expecting, we just print it literally. Note that we must handle non-Rust
    // symbols because we could have any function in the backtrace.
    let inner;
    if s.len() > 4 && s.starts_with("_ZN") && s.ends_with('E') {
        inner = &s[3..s.len() - 1];
    } else if s.len() > 3 && s.starts_with("ZN") && s.ends_with('E') {
        // On Windows, dbghelp strips leading underscores, so we accept "ZN...E"
        // form too.
        inner = &s[2..s.len() - 1];
    } else if s.len() > 5 && s.starts_with("__ZN") && s.ends_with('E') {
        // On OSX, symbols are prefixed with an extra _
        inner = &s[4..s.len() - 1];
    } else {
        return Err(());
    }

    // only work with ascii text
    if inner.bytes().any(|c| c & 0x80 != 0) {
        return Err(());
    }

    let mut elements = 0;
    let mut chars = inner.chars().peekable();
    loop {
        let mut i = 0usize;
        while let Some(&c) = chars.peek() {
            if !c.is_digit(10) {
                break
            }
            chars.next();
            let next = i.checked_mul(10)
                .and_then(|i| i.checked_add(c as usize - '0' as usize));
            i = match next {
                Some(i) => i,
                None => {
                    return Err(());
                }
            };
        }

        if i == 0 {
            if !chars.next().is_none() {
                return Err(());
            }
            break;
        } else if chars.by_ref().take(i).count() != i {
            return Err(());
        } else {
            elements += 1;
        }
    }

    Ok(Demangle {
        inner: inner,
        elements: elements,
    })
}

// Rust hashes are hex digits with an `h` prepended.
fn is_rust_hash(s: &str) -> bool {
    s.starts_with('h') && s[1..].chars().all(|c| c.is_digit(16))
}

impl<'a> fmt::Display for Demangle<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Alright, let's do this.
        let mut inner = self.inner;
        for element in 0..self.elements {
            let mut rest = inner;
            while rest.chars().next().unwrap().is_digit(10) {
                rest = &rest[1..];
            }
            let i: usize = inner[..(inner.len() - rest.len())].parse().unwrap();
            inner = &rest[i..];
            rest = &rest[..i];
            // Skip printing the hash if alternate formatting
            // was requested.
            if f.alternate() && element+1 == self.elements && is_rust_hash(&rest) {
                break;
            }
            if element != 0 {
                f.write_str("::")?;
            }
            if rest.starts_with("_$") {
                rest = &rest[1..];
            }
            while !rest.is_empty() {
                if rest.starts_with('.') {
                    if let Some('.') = rest[1..].chars().next() {
                        f.write_str("::")?;
                        rest = &rest[2..];
                    } else {
                        f.write_str(".")?;
                        rest = &rest[1..];
                    }
                } else if rest.starts_with('$') {
                    macro_rules! demangle {
                        ($($pat:expr => $demangled:expr,)*) => ({
                            $(if rest.starts_with($pat) {
                                f.write_str($demangled)?;
                                rest = &rest[$pat.len()..];
                              } else)*
                            {
                                f.write_str(rest)?;
                                break;
                            }

                        })
                    }

                    // see src/librustc/back/link.rs for these mappings
                    demangle! {
                        "$SP$" => "@",
                        "$BP$" => "*",
                        "$RF$" => "&",
                        "$LT$" => "<",
                        "$GT$" => ">",
                        "$LP$" => "(",
                        "$RP$" => ")",
                        "$C$" => ",",

                        // in theory we can demangle any Unicode code point, but
                        // for simplicity we just catch the common ones.
                        "$u7e$" => "~",
                        "$u20$" => " ",
                        "$u27$" => "'",
                        "$u3d$" => "=",
                        "$u5b$" => "[",
                        "$u5d$" => "]",
                        "$u7b$" => "{",
                        "$u7d$" => "}",
                        "$u3b$" => ";",
                        "$u2b$" => "+",
                        "$u21$" => "!",
                        "$u22$" => "\"",
                    }
                } else {
                    let idx = match rest.char_indices().find(|&(_, c)| c == '$' || c == '.') {
                        None => rest.len(),
                        Some((i, _)) => i,
                    };
                    f.write_str(&rest[..idx])?;
                    rest = &rest[idx..];
                }
            }
        }

        Ok(())
    }
}
