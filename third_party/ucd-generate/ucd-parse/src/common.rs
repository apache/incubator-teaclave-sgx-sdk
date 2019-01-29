// This module defines various common things used throughout the UCD.

use std::char;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use regex::Regex;

use error::{Error, error_set_line};

/// Parse a particular file in the UCD into a sequence of rows.
///
/// The given directory should be the directory to the UCD.
pub fn parse<P, D>(
    ucd_dir: P,
) -> Result<Vec<D>, Error>
where P: AsRef<Path>, D: UcdFile
{
    let mut xs = vec![];
    for result in D::from_dir(ucd_dir)? {
        let x = result?;
        xs.push(x);
    }
    Ok(xs)
}

/// Parse a particular file in the UCD into a map from codepoint to the record.
///
/// The given directory should be the directory to the UCD.
pub fn parse_by_codepoint<P, D>(
    ucd_dir: P,
) -> Result<BTreeMap<Codepoint, D>, Error>
where P: AsRef<Path>, D: UcdFileByCodepoint
{
    let mut map = BTreeMap::new();
    for result in D::from_dir(ucd_dir)? {
        let x = result?;
        for cp in x.codepoints() {
            map.insert(cp, x.clone());
        }
    }
    Ok(map)
}

/// Parse a particular file in the UCD into a map from codepoint to all
/// records associated with that codepoint.
///
/// This is useful for files that have multiple records for each codepoint.
/// For example, the `NameAliases.txt` file lists multiple aliases for some
/// codepoints.
///
/// The given directory should be the directory to the UCD.
pub fn parse_many_by_codepoint<P, D>(
    ucd_dir: P,
) -> Result<BTreeMap<Codepoint, Vec<D>>, Error>
where P: AsRef<Path>, D: UcdFileByCodepoint
{
    let mut map = BTreeMap::new();
    for result in D::from_dir(ucd_dir)? {
        let x = result?;
        for cp in x.codepoints() {
            map.entry(cp).or_insert(vec![]).push(x.clone());
        }
    }
    Ok(map)
}

/// A helper function for parsing a common record format that associates one
/// or more codepoints with a string value.
pub fn parse_codepoint_association<'a>(
    line: &'a str,
) -> Result<(Codepoints, &'a str), Error>
{
    lazy_static! {
        static ref PARTS: Regex = Regex::new(
            r"(?x)
            ^
            \s*(?P<codepoints>[^\s;]+)\s*;
            \s*(?P<property>[^;\x23]+)\s*
            "
        ).unwrap();
    };

    let caps = match PARTS.captures(line.trim()) {
        Some(caps) => caps,
        None => return err!("invalid PropList line: '{}'", line),
    };
    let property = match caps.name("property") {
        Some(property) => property.as_str().trim(),
        None => return err!(
            "could not find property name in PropList line: '{}'", line),
    };
    Ok((caps["codepoints"].parse()?, property))
}

/// A helper function for parsing a sequence of space separated codepoints.
/// The sequence is permitted to be empty.
pub fn parse_codepoint_sequence(s: &str) -> Result<Vec<Codepoint>, Error> {
    let mut cps = vec![];
    for cp in s.trim().split_whitespace() {
        cps.push(cp.parse()?);
    }
    Ok(cps)
}

/// A helper function for parsing a single test for the various break
/// algorithms.
///
/// Upon success, this returns the UTF-8 encoded groups of codepoints along
/// with the comment associated with the test. The comment is a human readable
/// description of the test that may prove useful for debugging.
pub fn parse_break_test(line: &str) -> Result<(Vec<String>, String), Error> {
    lazy_static! {
        static ref PARTS: Regex = Regex::new(
            r"(?x)
            ^
            (?:÷|×)
            (?P<groups>(?:\s[0-9A-Fa-f]{4,5}\s(?:÷|×))+)
            \s+
            \#(?P<comment>.+)
            $
            "
        ).unwrap();

        static ref GROUP: Regex = Regex::new(
            r"(?x)
            (?P<codepoint>[0-9A-Fa-f]{4,5})\s(?P<kind>÷|×)
            "
        ).unwrap();
    }

    let caps = match PARTS.captures(line.trim()) {
        Some(caps) => caps,
        None => return err!("invalid break test line: '{}'", line),
    };
    let comment = caps["comment"].trim().to_string();

    let mut groups = vec![];
    let mut cur = String::new();
    for cap in GROUP.captures_iter(&caps["groups"]) {
        let cp: Codepoint = cap["codepoint"].parse()?;
        let ch = match cp.scalar() {
            Some(ch) => ch,
            None => return err!(
                "invalid codepoint '{:X}' in line: '{}'", cp.value(), line
            ),
        };
        cur.push(ch);
        if &cap["kind"] == "÷" {
            groups.push(cur);
            cur = String::new();
        }
    }
    Ok((groups, comment))
}

/// Describes a single UCD file.
pub trait UcdFile:
    Clone + fmt::Debug + Default + Eq + FromStr<Err=Error> + PartialEq
{
    /// The file path corresponding to this file, relative to the UCD
    /// directory.
    fn relative_file_path() -> &'static Path;

    /// The full file path corresponding to this file given the UCD directory
    /// path.
    fn file_path<P: AsRef<Path>>(ucd_dir: P) -> PathBuf {
        ucd_dir.as_ref().join(Self::relative_file_path())
    }

    /// Create an iterator over each record in this UCD file.
    ///
    /// The parameter should correspond to the directory containing the UCD.
    fn from_dir<P: AsRef<Path>>(
        ucd_dir: P,
    ) -> Result<UcdLineParser<File, Self>, Error> {
        UcdLineParser::from_path(Self::file_path(ucd_dir))
    }
}

/// Describes a single UCD file where every record in the file is associated
/// with one or more codepoints.
pub trait UcdFileByCodepoint: UcdFile {
    /// Returns the codepoints associated with this record.
    fn codepoints(&self) -> CodepointIter;
}

/// A line oriented parser for a particular UCD file.
///
/// Callers can build a line parser via the
/// [`UcdFile::from_dir`](trait.UcdFile.html) method.
///
/// The `R` type parameter refers to the underlying `io::Read` implementation
/// from which the UCD data is read.
///
/// The `D` type parameter refers to the type of the record parsed out of each
/// line.
#[derive(Debug)]
pub struct UcdLineParser<R, D> {
    rdr: io::BufReader<R>,
    line: String,
    line_number: u64,
    _data: PhantomData<D>,
}

impl<D> UcdLineParser<File, D> {
    /// Create a new parser from the given file path.
    pub(crate) fn from_path<P: AsRef<Path>>(
        path: P,
    ) -> Result<UcdLineParser<File, D>, Error> {
        let file = File::open(path)?;
        Ok(UcdLineParser::new(file))
    }
}

impl<R: io::Read, D> UcdLineParser<R, D> {
    /// Create a new parser that parses the reader given.
    ///
    /// The type of data parsed is determined when the `parse_next` function
    /// is called by virtue of the type requested.
    ///
    /// Note that the reader is buffered internally, so the caller does not
    /// need to provide their own buffering.
    pub(crate) fn new(rdr: R) -> UcdLineParser<R, D> {
        UcdLineParser {
            rdr: io::BufReader::new(rdr),
            line: String::new(),
            line_number: 0,
            _data: PhantomData,
        }
    }
}

impl<R: io::Read, D: FromStr<Err=Error>> Iterator for UcdLineParser<R, D> {
    type Item = Result<D, Error>;

    fn next(&mut self) -> Option<Result<D, Error>> {
        loop {
            self.line_number += 1;
            self.line.clear();
            let n = match self.rdr.read_line(&mut self.line) {
                Err(err) => return Some(Err(Error::from(err))),
                Ok(n) => n,
            };
            if n == 0 {
                return None;
            }
            if !self.line.starts_with('#') && !self.line.trim().is_empty() {
                break;
            }
        }
        let line_number = self.line_number;
        Some(self.line.parse().map_err(|mut err| {
            error_set_line(&mut err, Some(line_number));
            err
        }))
    }
}

/// A representation of either a single codepoint or a range of codepoints.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Codepoints {
    /// A single codepoint.
    Single(Codepoint),
    /// A range of codepoints.
    Range(CodepointRange),
}

impl Default for Codepoints {
    fn default() -> Codepoints {
        Codepoints::Single(Codepoint::default())
    }
}

impl IntoIterator for Codepoints {
    type IntoIter = CodepointIter;
    type Item = Codepoint;

    fn into_iter(self) -> CodepointIter {
        match self {
            Codepoints::Single(x) => x.into_iter(),
            Codepoints::Range(x) => x.into_iter(),
        }
    }
}

impl FromStr for Codepoints {
    type Err = Error;

    fn from_str(s: &str) -> Result<Codepoints, Error> {
        if s.contains("..") {
            CodepointRange::from_str(s).map(Codepoints::Range)
        } else {
            Codepoint::from_str(s).map(Codepoints::Single)
        }
    }
}

impl fmt::Display for Codepoints {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Codepoints::Single(ref x) => x.fmt(f),
            Codepoints::Range(ref x) => x.fmt(f),
        }
    }
}

impl PartialEq<u32> for Codepoints {
    fn eq(&self, other: &u32) -> bool {
        match *self {
            Codepoints::Single(ref x) => x == other,
            Codepoints::Range(ref x) => x == &(*other, *other),
        }
    }
}

impl PartialEq<Codepoint> for Codepoints {
    fn eq(&self, other: &Codepoint) -> bool {
        match *self {
            Codepoints::Single(ref x) => x == other,
            Codepoints::Range(ref x) => x == &(*other, *other),
        }
    }
}

impl PartialEq<(u32, u32)> for Codepoints {
    fn eq(&self, other: &(u32, u32)) -> bool {
        match *self {
            Codepoints::Single(ref x) => &(x.value(), x.value()) == other,
            Codepoints::Range(ref x) => x == other,
        }
    }
}

impl PartialEq<(Codepoint, Codepoint)> for Codepoints {
    fn eq(&self, other: &(Codepoint, Codepoint)) -> bool {
        match *self {
            Codepoints::Single(ref x) => &(*x, *x) == other,
            Codepoints::Range(ref x) => x == other,
        }
    }
}

/// A range of Unicode codepoints. The range is inclusive; both ends of the
/// range are guaranteed to be valid codepoints.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct CodepointRange {
    /// The start of the codepoint range.
    pub start: Codepoint,
    /// The end of the codepoint range.
    pub end: Codepoint,
}

impl IntoIterator for CodepointRange {
    type IntoIter = CodepointIter;
    type Item = Codepoint;

    fn into_iter(self) -> CodepointIter {
        CodepointIter { next: self.start.value(), range: self }
    }
}

impl FromStr for CodepointRange {
    type Err = Error;

    fn from_str(s: &str) -> Result<CodepointRange, Error> {
        lazy_static! {
            static ref PARTS: Regex = Regex::new(
                r"^(?P<start>[A-Z0-9]+)\.\.(?P<end>[A-Z0-9]+)$"
            ).unwrap();
        }
        let caps = match PARTS.captures(s) {
            Some(caps) => caps,
            None => return err!("invalid codepoint range: '{}'", s),
        };
        let start = caps["start"].parse().or_else(|err| {
            err!("failed to parse '{}' as a codepoint range: {}", s, err)
        })?;
        let end = caps["end"].parse().or_else(|err| {
            err!("failed to parse '{}' as a codepoint range: {}", s, err)
        })?;
        Ok(CodepointRange { start: start, end: end })
    }
}

impl fmt::Display for CodepointRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl PartialEq<(u32, u32)> for CodepointRange {
    fn eq(&self, other: &(u32, u32)) -> bool {
        &(self.start.value(), self.end.value()) == other
    }
}

impl PartialEq<(Codepoint, Codepoint)> for CodepointRange {
    fn eq(&self, other: &(Codepoint, Codepoint)) -> bool {
        &(self.start, self.end) == other
    }
}

/// A single Unicode codepoint.
///
/// This type's string representation is a hexadecimal number. It is guaranteed
/// to be in the range `[0, 10FFFF]`.
///
/// Note that unlike Rust's `char` type, this may be a surrogate codepoint.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Codepoint(u32);

impl Codepoint {
    /// Create a new codepoint from a `u32`.
    ///
    /// If the given number is not a valid codepoint, then this returns an
    /// error.
    pub fn from_u32(n: u32) -> Result<Codepoint, Error> {
        if n > 0x10FFFF {
            err!("{:x} is not a valid Unicode codepoint", n)
        } else {
            Ok(Codepoint(n))
        }
    }

    /// Return the underlying `u32` codepoint value.
    pub fn value(self) -> u32 { self.0 }

    /// Attempt to convert this codepoint to a Unicode scalar value.
    ///
    /// If this is a surrogate codepoint, then this returns `None`.
    pub fn scalar(self) -> Option<char> { char::from_u32(self.0) }
}

impl IntoIterator for Codepoint {
    type IntoIter = CodepointIter;
    type Item = Codepoint;

    fn into_iter(self) -> CodepointIter {
        let range = CodepointRange { start: self, end: self };
        CodepointIter { next: self.value(), range: range }
    }
}

impl FromStr for Codepoint {
    type Err = Error;

    fn from_str(s: &str) -> Result<Codepoint, Error> {
        match u32::from_str_radix(s, 16) {
            Ok(n) => Codepoint::from_u32(n),
            Err(err) => {
                return err!(
                    "failed to parse '{}' as a hexadecimal codepoint: {}",
                    s, err);
            }
        }
    }
}

impl fmt::Display for Codepoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X}", self.0)
    }
}

impl PartialEq<u32> for Codepoint {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Codepoint> for u32 {
    fn eq(&self, other: &Codepoint) -> bool {
        *self == other.0
    }
}

/// An iterator over a range of Unicode codepoints.
#[derive(Debug)]
pub struct CodepointIter {
    next: u32,
    range: CodepointRange,
}

impl Iterator for CodepointIter {
    type Item = Codepoint;

    fn next(&mut self) -> Option<Codepoint> {
        if self.next > self.range.end.value() {
            return None;
        }
        let current = self.next;
        self.next += 1;
        Some(Codepoint::from_u32(current).unwrap())
    }
}
