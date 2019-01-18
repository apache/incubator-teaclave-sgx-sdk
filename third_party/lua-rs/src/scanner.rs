use std::prelude::v1::*;
use ::{Error, Result};
use bytes::{BufMut, BytesMut};
use state::State;
use std::f64;
use std::io::{BufRead, BufReader, Read};
use std::mem;
use std::u8;

/// END_OF_STREAM indicates that scanner has reach the end of stream.
const EOF: char = 0xFF as char;
const INIT: char = 0x0 as char;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    And,
    Break,
    Do,
    Else,
    Elseif,
    End,
    False,
    For,
    Function,
    If,
    In,
    Local,
    Nil,
    Not,
    Or,
    Repeat,
    Return,
    Then,
    True,
    Until,
    While,
    Concat,
    Dots,
    Eq,
    GE,
    LE,
    NE,
    EOF,
    Number(f64),
    Ident(String),
    String(String),
    Char(char),
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match *self {
            Token::Number(ff) => format!("{}", ff),
            Token::Ident(ref s) => s.clone(),
            Token::String(ref s) => s.clone(),
            Token::Char(ref c) => c.to_string(),
            _ => {
                let s = match *self {
                    Token::And => "and",
                    Token::Break => "break",
                    Token::Do => "do",
                    Token::Else => "else",
                    Token::Elseif => "elseif",
                    Token::End => "end",
                    Token::False => "false",
                    Token::For => "for",
                    Token::Function => "function",
                    Token::If => "if",
                    Token::In => "in",
                    Token::Local => "local",
                    Token::Nil => "nil",
                    Token::Not => "not",
                    Token::Or => "or",
                    Token::Repeat => "repeat",
                    Token::Return => "return",
                    Token::Then => "then",
                    Token::True => "true",
                    Token::Until => "until",
                    Token::While => "while",
                    Token::Concat => "..",
                    Token::Dots => "...",
                    Token::Eq => "==",
                    Token::GE => ">=",
                    Token::LE => "<=",
                    Token::NE => "~=",
                    Token::EOF => "<eof>",
                    _ => unreachable!()
                };
                s.to_string()
            }
        }
    }
}

fn is_new_line(c: char) -> bool {
    c == '\r' || c == '\n'
}

fn is_decimal(c: char) -> bool {
    '0' <= c && c <= '9'
}

fn is_hexadecimal(c: char) -> bool {
    ('0' <= c && c <= '9') || ('a' <= c && c <= 'f') || ('A' <= c && c <= 'F')
}

#[derive(Debug)]
pub struct Scanner<R> {
    reader: BufReader<R>,
    buffer: BytesMut,
    line_number: u32,
    current: char,
}

impl<R: Read> Scanner<R> {
    pub fn new(reader: BufReader<R>) -> Scanner<R> {
        Scanner {
            reader,
            buffer: BytesMut::new(),
            line_number: 1,
            current: INIT,
        }
    }

    pub fn line_number(&self) -> u32 {
        self.line_number
    }

    pub fn scan(&mut self) -> Result<Token> {
        loop {
            //println!("{:?}", self.current);
            match self.current {
                EOF => return Ok(Token::EOF),
                INIT => self.advance(),
                ' ' => self.advance(),
                '\t' => self.advance(),
                '\u{013}' => self.advance(),/*vertical tab character*/
                '\u{014}' => self.advance(),/*form feed*/
                '\r' | '\n' => self.incr_line_number(),
                '-' => {
                    self.advance();
                    if self.current != '-' {
                        return Ok(Token::Char('-'));
                    }

                    self.advance();
                    if self.current == '[' {
                        let sep = self.skip_sep();
                        if sep >= 0 {
                            self.read_multi_line(true, sep)?;
                            continue;
                        }
                        self.buffer.clear();
                    }

                    while !is_new_line(self.current) && self.current != EOF {
                        self.advance();
                    }
                }
                '[' => {
                    let sep = self.skip_sep();
                    if sep >= 0 {
                        return Ok(Token::String(self.read_multi_line(false, sep)?));
                    }
                    self.buffer.clear();
                    if sep == -1 {
                        return Ok(Token::Char('['));
                    }
                    return Err(Error::LexicalError("invalid long string delimiter".to_string()));
                }
                '=' => {
                    self.advance();
                    if self.current != '=' {
                        return Ok(Token::Char('='));
                    }
                    self.advance();
                    return Ok(Token::Eq);
                }
                '<' => {
                    self.advance();
                    if self.current != '=' {
                        return Ok(Token::Char('<'));
                    }
                    self.advance();
                    return Ok(Token::LE);
                }
                '>' => {
                    self.advance();
                    if self.current != '=' {
                        return Ok(Token::Char('>'));
                    }
                    self.advance();
                    return Ok(Token::GE);
                }
                '~' => {
                    self.advance();
                    if self.current != '=' {
                        return Ok(Token::Char('~'));
                    }
                    self.advance();
                    return Ok(Token::NE);
                }
                '"' | '\'' => return self.read_string(),
                '.' => {
                    self.save_and_advance();
                    if self.check_next(".") {
                        let t = if self.check_next(".") {
                            Token::Dots
                        } else {
                            Token::Concat
                        };
                        self.buffer.clear();
                        return Ok(t);
                    } else if !self.current.is_ascii_digit() {
                        self.buffer.clear();
                        return Ok(Token::Char('.'));
                    } else {
                        return self.read_number();
                    }
                }
                _ => {
                    let c = self.current;
                    if c.is_digit(10) {
                        return self.read_number();
                    } else if c == '_' || c.is_ascii_alphabetic() {
                        loop {
                            self.save_and_advance();
                            if self.current != '_' && !self.current.is_ascii_alphanumeric() {
                                break;
                            }
                        }
                        return self.reserved_or_name();
                    }
                    self.advance();
                    return Ok(Token::Char(c));
                }
            }
        }
        unreachable!()
    }

    fn advance(&mut self) {
        let c: char = match self.reader.fill_buf() {
            Ok(ref buf) if buf.len() > 0 => {
                buf[0] as char
            }
            _ => EOF,
        };

        println!("Advanced: {}", c);

        if c != EOF {
            self.reader.consume(1)
        }

        self.current = c;
    }

    fn save(&mut self, c: char) {
        self.buffer.reserve(1);
        self.buffer.put(c as u8)
    }

    fn save_and_advance(&mut self) {
        let c = self.current;
        self.save(c);
        self.advance();
    }

    fn advance_and_save(&mut self, c: char) {
        self.advance();
        self.save(c);
    }

    fn incr_line_number(&mut self) {
        let old = self.current;
        debug_assert!(is_new_line(old));

        self.advance();
        if is_new_line(self.current) && self.current != old {
            self.advance();
        }

        self.line_number = self.line_number + 1;
        // TODO: check lines too many?
    }

    fn skip_sep(&mut self) -> isize {
        let mut count: isize = 0;
        let c = self.current;
        debug_assert!(c == '[' || c == ']');
        self.save_and_advance();
        while self.current == '=' {
            self.save_and_advance();
            count += 1;
        }
        if self.current == c {
            count
        } else {
            -count - 1
        }
    }

    fn buf_string(&self) -> Result<String> {
        let buf_len = self.buffer.len();
        if buf_len > 0 {
            Ok(String::from_utf8(self.buffer[..].to_vec())?)
        } else {
            Ok(String::new())
        }
    }

    fn read_multi_line(&mut self, is_comment: bool, sep: isize) -> Result<String> {
        self.save_and_advance();
        if is_new_line(self.current) {
            self.incr_line_number();
        }
        loop {
            match self.current {
                EOF => {
                    if is_comment {
                        return Err(Error::LexicalError("unfinished long comment".to_string()));
                    } else {
                        return Err(Error::LexicalError("unfinished long string".to_string()));
                    }
                }
                ']' => {
                    let sep2 = self.skip_sep();
                    let mut ret = String::new();
                    if sep == sep2 {
                        self.save_and_advance();
                        if !is_comment {
                            let buf_len = self.buffer.len();
                            ret = String::from_utf8(self.buffer[(2 + sep) as usize..(buf_len - 2)].to_vec())?;
                        }
                        self.buffer.clear();
                        return Ok(ret);
                    }
                }
                '\r' => self.advance(),
                '\n' => {
                    let current = self.current;
                    self.save(current);
                    self.incr_line_number();
                }

                _ => {
                    if !is_comment {
                        let current = self.current;
                        self.save(current);
                    }
                    self.advance();
                }
            }
        }
        unreachable!()
    }

    fn read_string(&mut self) -> Result<Token> {
        let delimiter = self.current;
        self.advance();
        while self.current != delimiter {
            match self.current {
                EOF | '\n' | '\r' => return Err(Error::LexicalError("unfinished string".to_string())),
                '\\' => {
                    self.advance();
                    let current = self.current;
                    match current {
                        /// Escape charactors
                        /// \a   U+0007 alert or bell
                        /// \b   U+0008 backspace
                        /// \f   U+000C form feed
                        /// \n   U+000A line feed or newline
                        /// \r   U+000D carriage return
                        /// \t   U+0009 horizontal tab
                        /// \v   U+000b vertical tab
                        /// \\   U+005c backslash
                        /// \'   U+0027 single quote  (valid escape only within rune literals)
                        /// \"   U+0022 double quote  (valid escape only within string literals)
                        'a' => self.advance_and_save('\u{0007}'),
                        'b' => self.advance_and_save('\u{0008}'),
                        'f' => self.advance_and_save('\u{000C}'),
                        'n' => self.advance_and_save('\u{000A}'),
                        'r' => self.advance_and_save('\u{000D}'),
                        't' => self.advance_and_save('\u{0009}'),
                        'v' => self.advance_and_save('\u{000b}'),
                        '\\' => self.advance_and_save('\u{005c}'),
                        '\'' => self.advance_and_save('\u{0027}'),
                        '"' => self.advance_and_save('\u{0022}'),
                        _ if current == EOF => {} // do nothing
                        _ if is_new_line(current) => {
                            self.incr_line_number();
                            self.save('\n');
                        }
                        _ if current == 'x' => {
                            let hex_esc = self.read_hex_escape()?;
                            self.save(hex_esc);
                        }
                        _ => {
                            if !is_decimal(current) {
                                return Err(Error::LexicalError("invalid escape sequence".to_string()));
                            }
                            let dec_esc = self.read_decimal_escape()?;
                            self.save(dec_esc);
                        }
                    }
                }
                _ => {
                    self.save_and_advance();
                }
            }
        }

        self.advance();
        let ret = self.buf_string()?;
        self.buffer.clear();
        return Ok(Token::String(ret));
    }

    fn read_hex_escape(&mut self) -> Result<char> {
        self.advance();
        let mut i = 1;
        let mut c = self.current;
        let mut r: u8 = 0;
        loop {
            if i > 2 {
                break;
            }
            let mut cvalue = c as u8;
            match c {
                _ if '0' <= c && c <= '9' => cvalue = cvalue - ('0' as u8),
                _ if 'a' <= c && c <= 'z' => cvalue = cvalue - ('a' as u8) + 10,
                _ if 'A' <= c && c <= 'z' => cvalue = cvalue - ('A' as u8) + 10,
                _ => return Err(Error::LexicalError("hexadecimal digit expected".to_string()))
            }

            self.advance();
            i = i + 1;
            c = self.current;
            r = r * 16 + cvalue;
        }

        Ok(r as char)
    }

    fn read_hexadecimal(&mut self, x: f64) -> (f64, char, isize) {
        let (mut c, mut n) = (self.current, x);
        if !is_hexadecimal(c) {
            return (n, c, 0);
        }
        let mut count: isize = 0;
        loop {
            let mut cvalue = c as u32;
            match c {
                _ if '0' <= c && c <= '9' => cvalue = cvalue - ('0' as u32),
                _ if 'a' <= c && c <= 'z' => cvalue = cvalue - ('a' as u32) + 10,
                _ if 'A' <= c && c <= 'z' => cvalue = cvalue - ('A' as u32) + 10,
                _ => break
            }
            self.advance();
            n = n * 16.0 + (cvalue as f64);
            c = self.current;
            count = count + 1;
        }
        (n, c, count)
    }

    fn read_decimal_escape(&mut self) -> Result<char> {
        let mut i = 1;
        let mut c = self.current;
        let mut r: u32 = 0;
        loop {
            if i > 2 || !is_decimal(c) {
                break;
            }
            let mut cvalue = c as u32;
            r = r * 10 + (cvalue - ('0' as u32));
            self.advance();
            c = self.current;
            i = i + 1;
        }
        if r > (u8::MAX as u32) {
            return Err(Error::LexicalError("decimal escape too large".to_string()));
        }
        Ok(r as u8 as char)
    }

    fn read_digits(&mut self) -> char {
        loop {
            if !is_decimal(self.current) {
                break;
            }
            self.save_and_advance();
        }
        self.current
    }

    fn check_next(&mut self, s: &str) -> bool {
        if self.current == INIT || !s.contains(self.current) {
            return false;
        }
        self.save_and_advance();
        true
    }

    fn read_number(&mut self) -> Result<Token> {
        let current = self.current;
        debug_assert!(is_decimal(current));
        self.save_and_advance();

        // hexadecimal
        if current == '0' && self.check_next("Xx") {
            let prefix = self.buf_string()?;
            debug_assert!(&prefix == "0x" || &prefix == "0X");
            self.buffer.clear();

            let mut exponent: isize = 0;
            let (mut fraction, mut latest_char, mut count) = self.read_hexadecimal(0.0);

            if latest_char == '.' {
                self.advance();
                let (frac, lchar, exp) = self.read_hexadecimal(fraction);
                fraction = frac;
                latest_char = lchar;
                exponent = exp;
            }

            if count == 0 && exponent == 0 {
                return Err(Error::LexicalError("malformed number".to_string()));
            }

            exponent = exponent * -4;
            if latest_char == 'p' || latest_char == 'P' {
                self.advance();
                let mut negative_exp = false;
                let c = self.current;
                if c == '+' || c == '-' {
                    negative_exp = c == '-';
                    self.advance();
                }

                if is_decimal(self.current) {
                    return Err(Error::LexicalError("malformed number".to_string()));
                }

                self.read_digits();
                let digits = self.buf_string()?;
                let number = match digits.parse::<isize>() {
                    Ok(number) => number,
                    _ => return Err(Error::LexicalError("malformed number".to_string()))
                };

                exponent = if negative_exp {
                    exponent - number
                } else {
                    exponent + number
                };

                self.buffer.clear();
            }

            return Ok(Token::Number(fraction * (exponent as f64).exp2()));
        }

        let mut latest_char = self.read_digits();
        if latest_char == '.' {
            self.save_and_advance();
            latest_char = self.read_digits();
        }

        if latest_char == 'e' || latest_char == 'E' {
            self.save_and_advance();
            if self.current == '+' || self.current == '-' {
                self.save_and_advance();
            }
            self.read_digits();
        }

        let mut s = self.buf_string()?;
        if s.starts_with("0") {
            let r = s.trim_left_matches("0").to_string();
            if r.len() < 1 || r == "" || !is_decimal(r.as_bytes()[0] as char) {
                s = format!("0{}", r);
            }
        }

        let number = match s.parse::<f64>() {
            Ok(n) => n,
            _ => return Err(Error::LexicalError("malformed number".to_string()))
        };

        self.buffer.clear();
        Ok(Token::Number(number))
    }

    /// Reserved words
    ///
    /// ```
    /// /* terminal symbols denoted by reserved words */
    /// ["and", "break", "do", "else", "elseif",
    /// "end", "false", "for", "function", "if",
    /// "in", "local", "nil", "not", "or", "repeat",
    /// "return", "then", "true", "until", "while",
    /// /* other terminal symbols */
    /// "..", "...", "==", ">=", "<=", "~=", "<eof>",
    /// "<number>", "<name>", "<string>"];
    /// ```
    fn reserved_or_name(&mut self) -> Result<Token> {
        let s = self.buf_string()?;
        debug_assert!(s.len() > 0);

        let t = match s.as_str() {
            "and" => Token::And,
            "break" => Token::Break,
            "do" => Token::Do,
            "else" => Token::Else,
            "elseif" => Token::Elseif,
            "end" => Token::End,
            "false" => Token::False,
            "for" => Token::For,
            "function" => Token::Function,
            "if" => Token::If,
            "in" => Token::In,
            "local" => Token::Local,
            "nil" => Token::Nil,
            "not" => Token::Not,
            "or" => Token::Or,
            "repeat" => Token::Repeat,
            "return" => Token::Return,
            "then" => Token::Then,
            "true" => Token::True,
            "until" => Token::Until,
            "while" => Token::While,
            _ => Token::Ident(s),
        };

        self.buffer.clear();
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use super::*;

    #[test]
    fn to_string() {
        let tests = [
            (Token::And, "and"),
            (Token::Break, "break"),
            (Token::Do, "do"),
            (Token::Else, "else"),
            (Token::Elseif, "elseif"),
            (Token::End, "end"),
            (Token::False, "false"),
            (Token::For, "for"),
            (Token::Function, "function"),
            (Token::If, "if"),
            (Token::In, "in"),
            (Token::Local, "local"),
            (Token::Nil, "nil"),
            (Token::Not, "not"),
            (Token::Or, "or"),
            (Token::Repeat, "repeat"),
            (Token::Return, "return"),
            (Token::Then, "then"),
            (Token::True, "true"),
            (Token::Until, "until"),
            (Token::While, "while"),
            (Token::Concat, ".."),
            (Token::Dots, "..."),
            (Token::Eq, "=="),
            (Token::GE, ">="),
            (Token::LE, "<="),
            (Token::NE, "~="),
            (Token::EOF, "<eof>"),
            (Token::Number(1.0), "1"),
            (Token::Number(1.2), "1.2"),
            (Token::Ident("example".to_string()), "example"),
            (Token::String("example".to_string()), "example"),
            (Token::Char('e'), "e"),
        ];

        for (t, exp) in tests.iter() {
            assert_eq!(t.to_string(), exp.to_string());
        }
    }

    #[test]
    fn is_fns() {
        assert!(is_new_line('\r'));
        assert!(is_new_line('\n'));
        assert!(!is_new_line('x'));

        assert!(is_decimal('0'));
        assert!(is_decimal('1'));
        assert!(is_decimal('9'));
        assert!(!is_decimal('x'));

        assert!(is_hexadecimal('0'));
        assert!(is_hexadecimal('1'));
        assert!(is_hexadecimal('a'));
        assert!(is_hexadecimal('f'));
        assert!(is_hexadecimal('A'));
        assert!(is_hexadecimal('F'));
        assert!(!is_hexadecimal('x'));
        assert!(!is_hexadecimal('X'));
    }

    #[test]
    fn scan() {
        let buf = b"x>=y\nx = 5";

        let mut scanner = Scanner::new(BufReader::new(&buf[..]));

        let res = scanner.scan();
        assert!(res.is_ok());
        println!("res: {:?}", res);
        assert_eq!(res.unwrap(), Token::Ident("x".to_string()));

        let res = scanner.scan();
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Token::GE);

        let res = scanner.scan();
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Token::Ident("y".to_string()));

        assert_eq!(scanner.line_number(), 1);

        let res = scanner.scan();
        assert_eq!(scanner.line_number(), 2);
    }
}

