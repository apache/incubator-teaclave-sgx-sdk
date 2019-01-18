use std::ffi::OsStr;
use std::ops;

use clap;

use error::Result;
use util::Filter;
use writer::{Writer, WriterBuilder};

/// Wraps clap matches and provides convenient accessors to various parameters.
pub struct ArgMatches<'a>(&'a clap::ArgMatches<'a>);

impl<'a> ops::Deref for ArgMatches<'a> {
    type Target = clap::ArgMatches<'a>;
    fn deref(&self) -> &clap::ArgMatches<'a> { &self.0 }
}

impl<'a> ArgMatches<'a> {
    pub fn new(matches: &'a clap::ArgMatches<'a>) -> ArgMatches<'a> {
        ArgMatches(matches)
    }

    pub fn ucd_dir(&self) -> Result<&OsStr> {
        match self.value_of_os("ucd-dir") {
            Some(x) => Ok(x),
            None => err!("missing UCD directory"),
        }
    }

    pub fn writer(&self, name: &str) -> Result<Writer> {
        let mut builder = WriterBuilder::new(name);
        builder
            .columns(79)
            .char_literals(self.is_present("chars"))
            .trie_set(self.is_present("trie-set"));
        if let Some(p) = self.value_of_os("dfa-dir") {
            return builder.from_dfa_dir(p);
        }
        match self.value_of_os("fst-dir") {
            None => Ok(builder.from_stdout()),
            Some(x) => builder.from_fst_dir(x),
        }
    }

    pub fn dfa_writer(&self, name: &str) -> Result<Writer> {
        let mut builder = WriterBuilder::new(name);
        builder
            .columns(79)
            .char_literals(self.is_present("chars"))
            .trie_set(self.is_present("trie-set"));
        if let Some(p) = self.value_of_os("dfa-dir") {
            builder.from_dfa_dir(p)
        } else {
            err!("missing DFA directory")
        }
    }

    pub fn name(&self) -> &str {
        self.value_of("name").expect("the name of the table")
    }

    /// Create a new include/exclude filter command line arguments.
    ///
    /// The given canonicalization function is applied to each element in
    /// each of the include/exclude lists provided by the end user.
    pub fn filter<F: FnMut(&str) -> Result<String>>(
        &self,
        mut canonicalize: F,
    ) -> Result<Filter> {
        Filter::new(
            self.value_of_lossy("include").map(|s| s.to_string()),
            self.value_of_lossy("exclude").map(|s| s.to_string()),
            |name| canonicalize(name))
    }
}
