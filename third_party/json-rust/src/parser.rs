// HERE BE DRAGONS!
// ================
//
// Making a fast parser is hard. This is a _not so naive_ implementation of
// recursive descent that does almost nothing. _There is no backtracking_, the
// whole parsing is 100% predictive, even though it's not BNF, and will have
// linear performance based on the length of the source!
//
// There is a lot of macros here! Like, woah! This is mostly due to the fact
// that Rust isn't very cool about optimizing inlined functions that return
// a `Result` type. Since different functions will have different `Result`
// signatures, the `try!` macro will always have to repackage our results.
// With macros those issues don't exist, the macro will return an unpackaged
// result - whatever it is - and if we ever stumble upon the error, we can
// return an `Err` without worrying about the exact signature of `Result`.
//
// This makes for some ugly code, but it is faster. Hopefully in the future
// with MIR support the compiler will get smarter about this.

use std::{ str, slice };
use object::Object;
use number::Number;
use { JsonValue, Error, Result };
use std::vec::Vec;

// This is not actual max precision, but a threshold at which number parsing
// kicks into checked math.
const MAX_PRECISION: u64 = 576460752303423500;


// How many nested Objects/Arrays are allowed to be parsed
const DEPTH_LIMIT: usize = 512;


// The `Parser` struct keeps track of indexing over our buffer. All niceness
// has been abandoned in favor of raw pointer magic. Does that make you feel
// dirty? _Good._
struct Parser<'a> {
    // Helper buffer for parsing strings that can't be just memcopied from
    // the original source (escaped characters)
    buffer: Vec<u8>,

    // String slice to parse
    source: &'a str,

    // Byte pointer to the slice above
    byte_ptr: *const u8,

    // Current index
    index: usize,

    // Length of the source
    length: usize,
}


// Read a byte from the source.
// Will return an error if there are no more bytes.
macro_rules! expect_byte {
    ($parser:ident) => ({
        if $parser.is_eof() {
            return Err(Error::UnexpectedEndOfJson);
        }

        let ch = $parser.read_byte();
        $parser.bump();
        ch
    })
}


// Expect a sequence of specific bytes in specific order, error otherwise.
// This is useful for reading the 3 JSON identifiers:
//
// - "t" has to be followed by "rue"
// - "f" has to be followed by "alse"
// - "n" has to be followed by "ull"
//
// Anything else is an error.
macro_rules! expect_sequence {
    ($parser:ident, $( $ch:pat ),*) => {
        $(
            match expect_byte!($parser) {
                $ch => {},
                _   => return $parser.unexpected_character(),
            }
        )*
    }
}


// A drop in macro for when we expect to read a byte, but we don't care
// about any whitespace characters that might occur before it.
macro_rules! expect_byte_ignore_whitespace {
    ($parser:ident) => ({
        let mut ch = expect_byte!($parser);

        // Don't go straight for the loop, assume we are in the clear first.
        match ch {
            // whitespace
            9 ... 13 | 32 => {
                loop {
                    match expect_byte!($parser) {
                        9 ... 13 | 32 => {},
                        next          => {
                            ch = next;
                            break;
                        }
                    }
                }
            },
            _ => {}
        }

        ch
    })
}

// Expect to find EOF or just whitespaces leading to EOF after a JSON value
macro_rules! expect_eof {
    ($parser:ident) => ({
        while !$parser.is_eof() {
            match $parser.read_byte() {
                9 ... 13 | 32 => $parser.bump(),
                _             => {
                    $parser.bump();
                    return $parser.unexpected_character();
                }
            }
        }
    })
}

// Expect a particular byte to be next. Also available with a variant
// creates a `match` expression just to ease some pain.
macro_rules! expect {
    ($parser:ident, $byte:expr) => ({
        let ch = expect_byte_ignore_whitespace!($parser);

        if ch != $byte {
            return $parser.unexpected_character()
        }
    });

    {$parser:ident $(, $byte:pat => $then:expr )*} => ({
        let ch = expect_byte_ignore_whitespace!($parser);

        match ch {
            $(
                $byte => $then,
            )*
            _ => return $parser.unexpected_character()
        }

    })
}


// Look up table that marks which characters are allowed in their raw
// form in a string.
const QU: bool = false;  // double quote       0x22
const BS: bool = false;  // backslash          0x5C
const CT: bool = false;  // control character  0x00 ... 0x1F
const __: bool = true;

static ALLOWED: [bool; 256] = [
// 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
  CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
  CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
  __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
  __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];


// Expect a string. This is called after encountering, and consuming, a
// double quote character. This macro has a happy path variant where it
// does almost nothing as long as all characters are allowed (as described
// in the look up table above). If it encounters a closing quote without
// any escapes, it will use a slice straight from the source, avoiding
// unnecessary buffering.
macro_rules! expect_string {
    ($parser:ident) => ({
        let result: &str;
        let start = $parser.index;

        loop {
            let ch = expect_byte!($parser);
            if ALLOWED[ch as usize] {
                continue;
            }
            if ch == b'"' {
                unsafe {
                    let ptr = $parser.byte_ptr.offset(start as isize);
                    let len = $parser.index - 1 - start;
                    result = str::from_utf8_unchecked(slice::from_raw_parts(ptr, len));
                }
                break;
            }
            if ch == b'\\' {
                result = try!($parser.read_complex_string(start));
                break;
            }

            return $parser.unexpected_character();
        }

        result
    })
}


// Expect a number. Of some kind.
macro_rules! expect_number {
    ($parser:ident, $first:ident) => ({
        let mut num = ($first - b'0') as u64;

        let result: Number;

        // Cap on how many iterations we do while reading to u64
        // in order to avoid an overflow.
        loop {
            if num >= MAX_PRECISION {
                result = try!($parser.read_big_number(num));
                break;
            }

            if $parser.is_eof() {
                result = num.into();
                break;
            }

            let ch = $parser.read_byte();

            match ch {
                b'0' ... b'9' => {
                    $parser.bump();
                    num = num * 10 + (ch - b'0') as u64;
                },
                _             => {
                    let mut e = 0;
                    result = allow_number_extensions!($parser, num, e, ch);
                    break;
                }
            }
        }

        result
    })
}


// Invoked after parsing an integer, this will account for fractions and/or
// `e` notation.
macro_rules! allow_number_extensions {
    ($parser:ident, $num:ident, $e:ident, $ch:ident) => ({
        match $ch {
            b'.'        => {
                $parser.bump();
                expect_fraction!($parser, $num, $e)
            },
            b'e' | b'E' => {
                $parser.bump();
                try!($parser.expect_exponent($num, $e))
            },
            _  => $num.into()
        }
    });

    // Alternative variant that defaults everything to 0. This is actually
    // quite handy as the only number that can begin with zero, has to have
    // a zero mantissa. Leading zeroes are illegal in JSON!
    ($parser:ident) => ({
        if $parser.is_eof() {
            0.into()
        } else {
            let mut num = 0;
            let mut e = 0;
            let ch = $parser.read_byte();
            allow_number_extensions!($parser, num, e, ch)
        }
    })
}


// If a dot `b"."` byte has been read, start reading the decimal fraction
// of the number.
macro_rules! expect_fraction {
    ($parser:ident, $num:ident, $e:ident) => ({
        let result: Number;

        let ch = expect_byte!($parser);

        match ch {
            b'0' ... b'9' => {
                if $num < MAX_PRECISION {
                    $num = $num * 10 + (ch - b'0') as u64;
                    $e -= 1;
                } else {
                    match $num.checked_mul(10).and_then(|num| {
                        num.checked_add((ch - b'0') as u64)
                    }) {
                        Some(result) => {
                            $num = result;
                            $e -= 1;
                        },
                        None => {}
                    }
                }
            },
            _ => return $parser.unexpected_character()
        }

        loop {
            if $parser.is_eof() {
                result = unsafe { Number::from_parts_unchecked(true, $num, $e) };
                break;
            }
            let ch = $parser.read_byte();

            match ch {
                b'0' ... b'9' => {
                    $parser.bump();
                    if $num < MAX_PRECISION {
                        $num = $num * 10 + (ch - b'0') as u64;
                        $e -= 1;
                    } else {
                        match $num.checked_mul(10).and_then(|num| {
                            num.checked_add((ch - b'0') as u64)
                        }) {
                            Some(result) => {
                                $num = result;
                                $e -= 1;
                            },
                            None => {}
                        }
                    }
                },
                b'e' | b'E' => {
                    $parser.bump();
                    result = try!($parser.expect_exponent($num, $e));
                    break;
                }
                _ => {
                    result = unsafe { Number::from_parts_unchecked(true, $num, $e) };
                    break;
                }
            }
        }

        result
    })
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Parser {
            buffer: Vec::with_capacity(30),
            source: source,
            byte_ptr: source.as_ptr(),
            index: 0,
            length: source.len(),
        }
    }

    // Check if we are at the end of the source.
    #[inline(always)]
    fn is_eof(&mut self) -> bool {
        self.index == self.length
    }

    // Read a byte from the source. Note that this does not increment
    // the index. In few cases (all of them related to number parsing)
    // we want to peek at the byte before doing anything. This will,
    // very very rarely, lead to a situation where the same byte is read
    // twice, but since this operation is using a raw pointer, the cost
    // is virtually irrelevant.
    #[inline(always)]
    fn read_byte(&mut self) -> u8 {
        debug_assert!(self.index < self.length, "Reading out of bounds");

        unsafe { *self.byte_ptr.offset(self.index as isize) }
    }

    // Manually increment the index. Calling `read_byte` and then `bump`
    // is equivalent to consuming a byte on an iterator.
    #[inline(always)]
    fn bump(&mut self) {
        self.index = self.index.wrapping_add(1);
    }

    // So we got an unexpected character, now what? Well, figure out where
    // it is, and throw an error!
    fn unexpected_character<T: Sized>(&mut self) -> Result<T> {
        let at = self.index - 1;

        let ch = self.source[at..]
                     .chars()
                     .next()
                     .expect("Must have a character");

        let (lineno, col) = self.source[..at]
                                .lines()
                                .enumerate()
                                .last()
                                .unwrap_or((0, ""));

        let colno = col.chars().count();

        Err(Error::UnexpectedCharacter {
            ch: ch,
            line: lineno + 1,
            column: colno + 1,
        })
    }

    // Boring
    fn read_hexdec_digit(&mut self) -> Result<u32> {
        let ch = expect_byte!(self);
        Ok(match ch {
            b'0' ... b'9' => (ch - b'0'),
            b'a' ... b'f' => (ch + 10 - b'a'),
            b'A' ... b'F' => (ch + 10 - b'A'),
            _             => return self.unexpected_character(),
        } as u32)
    }

    // Boring
    fn read_hexdec_codepoint(&mut self) -> Result<u32> {
        Ok(
            try!(self.read_hexdec_digit()) << 12 |
            try!(self.read_hexdec_digit()) << 8  |
            try!(self.read_hexdec_digit()) << 4  |
            try!(self.read_hexdec_digit())
        )
    }

    // Oh look, some action. This method reads an escaped unicode
    // sequence such as `\uDEAD` from the string. Except `DEAD` is
    // not a valid codepoint, so it also needs to handle errors...
    fn read_codepoint(&mut self) -> Result<()> {
        let mut codepoint = try!(self.read_hexdec_codepoint());

        match codepoint {
            0x0000 ... 0xD7FF => {},
            0xD800 ... 0xDBFF => {
                codepoint -= 0xD800;
                codepoint <<= 10;

                expect_sequence!(self, b'\\', b'u');

                let lower = try!(self.read_hexdec_codepoint());

                if let 0xDC00 ... 0xDFFF = lower {
                    codepoint = (codepoint | lower - 0xDC00) + 0x010000;
                } else {
                    return Err(Error::FailedUtf8Parsing)
                }
            },
            0xE000 ... 0xFFFF => {},
            _ => return Err(Error::FailedUtf8Parsing)
        }

        match codepoint {
            0x0000 ... 0x007F => self.buffer.push(codepoint as u8),
            0x0080 ... 0x07FF => self.buffer.extend_from_slice(&[
                (((codepoint >> 6) as u8) & 0x1F) | 0xC0,
                ((codepoint        as u8) & 0x3F) | 0x80
            ]),
            0x0800 ... 0xFFFF => self.buffer.extend_from_slice(&[
                (((codepoint >> 12) as u8) & 0x0F) | 0xE0,
                (((codepoint >> 6)  as u8) & 0x3F) | 0x80,
                ((codepoint         as u8) & 0x3F) | 0x80
            ]),
            0x10000 ... 0x10FFFF => self.buffer.extend_from_slice(&[
                (((codepoint >> 18) as u8) & 0x07) | 0xF0,
                (((codepoint >> 12) as u8) & 0x3F) | 0x80,
                (((codepoint >> 6)  as u8) & 0x3F) | 0x80,
                ((codepoint         as u8) & 0x3F) | 0x80
            ]),
            _ => return Err(Error::FailedUtf8Parsing)
        }

        Ok(())
    }

    // What's so complex about strings you may ask? Not that much really.
    // This method is called if the `expect_string!` macro encounters an
    // escape. The added complexity is that it will have to use an internal
    // buffer to read all the escaped characters into, before finally
    // producing a usable slice. What it means it that parsing "foo\bar"
    // is whole lot slower than parsing "foobar", as the former suffers from
    // having to be read from source to a buffer and then from a buffer to
    // our target string. Nothing to be done about this, really.
    fn read_complex_string<'b>(&mut self, start: usize) -> Result<&'b str> {
        self.buffer.clear();
        let mut ch = b'\\';

        // TODO: Use fastwrite here as well
        self.buffer.extend_from_slice(self.source[start .. self.index - 1].as_bytes());

        loop {
            if ALLOWED[ch as usize] {
                self.buffer.push(ch);
                ch = expect_byte!(self);
                continue;
            }
            match ch {
                b'"'  => break,
                b'\\' => {
                    let escaped = expect_byte!(self);
                    let escaped = match escaped {
                        b'u'  => {
                            try!(self.read_codepoint());
                            ch = expect_byte!(self);
                            continue;
                        },
                        b'"'  |
                        b'\\' |
                        b'/'  => escaped,
                        b'b'  => 0x8,
                        b'f'  => 0xC,
                        b't'  => b'\t',
                        b'r'  => b'\r',
                        b'n'  => b'\n',
                        _     => return self.unexpected_character()
                    };
                    self.buffer.push(escaped);
                },
                _ => return self.unexpected_character()
            }
            ch = expect_byte!(self);
        }

        // Since the original source is already valid UTF-8, and `\`
        // cannot occur in front of a codepoint > 127, this is safe.
        Ok(unsafe {
            str::from_utf8_unchecked(
                // Because the buffer is stored on the parser, returning it
                // as a slice here freaks out the borrow checker. The compiler
                // can't know that the buffer isn't used till the result
                // of this function is long used and irrelevant. To avoid
                // issues here, we construct a new slice from raw parts, which
                // then has lifetime bound to the outer function scope instead
                // of the parser itself.
                slice::from_raw_parts(self.buffer.as_ptr(), self.buffer.len())
            )
        })
    }

    // Big numbers! If the `expect_number!` reaches a point where the decimal
    // mantissa could have overflown the size of u64, it will switch to this
    // control path instead. This method will pick up where the macro started,
    // but instead of continuing to read into the mantissa, it will increment
    // the exponent. Note that no digits are actually read here, as we already
    // exceeded the precision range of f64 anyway.
    fn read_big_number(&mut self, mut num: u64) -> Result<Number> {
        let mut e = 0i16;
        loop {
            if self.is_eof() {
                return Ok(unsafe { Number::from_parts_unchecked(true, num, e) });
            }
            let ch = self.read_byte();
            match ch {
                b'0' ... b'9' => {
                    self.bump();
                    match num.checked_mul(10).and_then(|num| {
                        num.checked_add((ch - b'0') as u64)
                    }) {
                        Some(result) => num = result,
                        None         => e = e.checked_add(1).ok_or_else(|| Error::ExceededDepthLimit)?,
                    }
                },
                b'.' => {
                    self.bump();
                    return Ok(expect_fraction!(self, num, e));
                },
                b'e' | b'E' => {
                    self.bump();
                    return self.expect_exponent(num, e);
                }
                _  => break
            }
        }

        Ok(unsafe { Number::from_parts_unchecked(true, num, e) })
    }

    // Called in the rare case that a number with `e` notation has been
    // encountered. This is pretty straight forward, I guess.
    fn expect_exponent(&mut self, num: u64, big_e: i16) -> Result<Number> {
        let mut ch = expect_byte!(self);
        let sign = match ch {
            b'-' => {
                ch = expect_byte!(self);
                -1
            },
            b'+' => {
                ch = expect_byte!(self);
                1
            },
            _    => 1
        };

        let mut e = match ch {
            b'0' ... b'9' => (ch - b'0') as i16,
            _ => return self.unexpected_character(),
        };

        loop {
            if self.is_eof() {
                break;
            }
            let ch = self.read_byte();
            match ch {
                b'0' ... b'9' => {
                    self.bump();
                    e = e.saturating_mul(10).saturating_add((ch - b'0') as i16);
                },
                _  => break
            }
        }

        Ok(unsafe { Number::from_parts_unchecked(true, num, big_e.saturating_add(e * sign)) })
    }

    // Parse away!
    fn parse(&mut self) -> Result<JsonValue> {
        let mut stack = Vec::with_capacity(3);
        let mut ch = expect_byte_ignore_whitespace!(self);

        'parsing: loop {
            let mut value = match ch {
                b'[' => {
                    ch = expect_byte_ignore_whitespace!(self);

                    if ch != b']' {
                        if stack.len() == DEPTH_LIMIT {
                            return Err(Error::ExceededDepthLimit);
                        }

                        stack.push(StackBlock(JsonValue::Array(Vec::with_capacity(2)), 0));
                        continue 'parsing;
                    }

                    JsonValue::Array(Vec::new())
                },
                b'{' => {
                    ch = expect_byte_ignore_whitespace!(self);

                    if ch != b'}' {
                        if stack.len() == DEPTH_LIMIT {
                            return Err(Error::ExceededDepthLimit);
                        }

                        let mut object = Object::with_capacity(3);

                        if ch != b'"' {
                            return self.unexpected_character()
                        }

                        let index = object.insert_index(expect_string!(self), JsonValue::Null);
                        expect!(self, b':');

                        stack.push(StackBlock(JsonValue::Object(object), index));

                        ch = expect_byte_ignore_whitespace!(self);

                        continue 'parsing;
                    }

                    JsonValue::Object(Object::new())
                },
                b'"' => expect_string!(self).into(),
                b'0' => JsonValue::Number(allow_number_extensions!(self)),
                b'1' ... b'9' => {
                    JsonValue::Number(expect_number!(self, ch))
                },
                b'-' => {
                    let ch = expect_byte!(self);
                    JsonValue::Number(- match ch {
                        b'0' => allow_number_extensions!(self),
                        b'1' ... b'9' => expect_number!(self, ch),
                        _    => return self.unexpected_character()
                    })
                }
                b't' => {
                    expect_sequence!(self, b'r', b'u', b'e');
                    JsonValue::Boolean(true)
                },
                b'f' => {
                    expect_sequence!(self, b'a', b'l', b's', b'e');
                    JsonValue::Boolean(false)
                },
                b'n' => {
                    expect_sequence!(self, b'u', b'l', b'l');
                    JsonValue::Null
                },
                _    => return self.unexpected_character()
            };

            'popping: loop {
                match stack.last_mut() {
                    None => {
                        expect_eof!(self);

                        return Ok(value);
                    },

                    Some(&mut StackBlock(JsonValue::Array(ref mut array), _)) => {
                        array.push(value);

                        ch = expect_byte_ignore_whitespace!(self);

                        match ch {
                            b',' => {
                                ch = expect_byte_ignore_whitespace!(self);

                                continue 'parsing;
                            },
                            b']' => {},
                            _    => return self.unexpected_character()
                        }
                    },

                    Some(&mut StackBlock(JsonValue::Object(ref mut object), ref mut index )) => {
                        object.override_at(*index, value);

                        ch = expect_byte_ignore_whitespace!(self);

                        match ch {
                            b',' => {
                                expect!(self, b'"');
                                *index = object.insert_index(expect_string!(self), JsonValue::Null);
                                expect!(self, b':');

                                ch = expect_byte_ignore_whitespace!(self);

                                continue 'parsing;
                            },
                            b'}' => {},
                            _    => return self.unexpected_character()
                        }
                    },

                    _ => unreachable!(),
                }

                value = match stack.pop() {
                    Some(StackBlock(value, _)) => value,
                    None                       => break 'popping
                }
            }
        }
    }
}

struct StackBlock(JsonValue, usize);

// All that hard work, and in the end it's just a single function in the API.
#[inline]
pub fn parse(source: &str) -> Result<JsonValue> {
    Parser::new(source).parse()
}
