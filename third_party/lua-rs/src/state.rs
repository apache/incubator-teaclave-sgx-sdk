#![allow(unused_variables)]

use std::prelude::v1::*;
use {parser, undump};
use ::Result;
use std::untrusted::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read};
use compiler;

/// A State is an opaque structure representing per thread Lua state.
#[derive(Debug)]
pub struct State {}

impl State {
    /// Creates a new thread running in a new, independent state.
    ///
    /// # Example
    ///
    /// ```
    /// use lua::state::State;
    ///
    /// let state = State::new();
    /// ```
    pub fn new() -> State {
        State {}
    }

    pub fn load_file(&mut self, path: &str) -> Result<()> {
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        self.load(reader, path)
    }

    pub fn load_string(&mut self, s: &str) -> Result<()> {
        let cursor = Cursor::new(s.as_bytes());
        let reader = BufReader::new(cursor);
        self.load(reader, "<string>")
    }

    /// Load lua chunk from reader
    /// Signature `\033Lua` indicates precompiled lua bytecode
    pub fn load<T: Read>(&mut self, mut reader: BufReader<T>, name: &str) -> Result<()> {
        let mut magic: u8 = 0;
        {
            let buf = reader.fill_buf()?;
            if buf.len() > 0 {
                magic = buf[0]
            }
        }
        let chunk = if magic == 033 {
            undump::undump(reader)?
        } else {
            let mut parser = parser::Parser::new(reader, name.to_string());
            let stmts = parser.parse()?;
            compiler::compile(stmts, name.to_string())?
        };

        // TODO: save chunk

        Ok(())
    }
}
