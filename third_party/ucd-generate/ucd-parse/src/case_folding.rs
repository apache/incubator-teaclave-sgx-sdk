use std::path::Path;
use std::str::FromStr;

use regex::Regex;

use common::{UcdFile, UcdFileByCodepoint, Codepoint, CodepointIter};
use error::Error;

/// A single row in the `CaseFolding.txt` file.
///
/// The contents of `CaseFolding.txt` are a convenience derived from both
/// `UnicodeData.txt` and `SpecialCasing.txt`.
///
/// Note that a single codepoint may be mapped multiple times. In particular,
/// a single codepoint might have distinct `CaseStatus::Simple` and
/// `CaseStatus::Full` mappings.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CaseFold {
    /// The codepoint that is being mapped.
    pub codepoint: Codepoint,
    /// The case status of this mapping.
    pub status: CaseStatus,
    /// The actual case mapping, which is more than one codepoint if this is
    /// a "full" mapping.
    pub mapping: Vec<Codepoint>,
}

impl UcdFile for CaseFold {
    fn relative_file_path() -> &'static Path {
        Path::new("CaseFolding.txt")
    }
}

impl UcdFileByCodepoint for CaseFold {
    fn codepoints(&self) -> CodepointIter {
        self.codepoint.into_iter()
    }
}

impl FromStr for CaseFold {
    type Err = Error;

    fn from_str(line: &str) -> Result<CaseFold, Error> {
        lazy_static! {
            static ref PARTS: Regex = Regex::new(
                r"(?x)
                ^
                \s*(?P<codepoint>[^\s;]+)\s*;
                \s*(?P<status>[^\s;]+)\s*;
                \s*(?P<mapping>[^;]+)\s*;
                "
            ).unwrap();
        };

        let caps = match PARTS.captures(line.trim()) {
            Some(caps) => caps,
            None => return err!("invalid CaseFolding line: '{}'", line),
        };
        let mut mapping = vec![];
        for cp in caps["mapping"].split_whitespace() {
            mapping.push(cp.parse()?);
        }
        Ok(CaseFold {
            codepoint: caps["codepoint"].parse()?,
            status: caps["status"].parse()?,
            mapping: mapping,
        })
    }
}

/// The status of a particular case mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaseStatus {
    /// Case mappings shared by both "simple" and "full" mappings.
    Common,
    /// A case mapping that changes the number of codepoints.
    Full,
    /// A case mapping that doesn't change the number of codepoints, when it
    /// differs from `Full`.
    Simple,
    /// Special cases (currently only for Turkic mappings) that are typically
    /// excluded by default. Special cases don't change the number of
    /// codepoints, but may changed the encoding (e.g., UTF-8) length in bytes.
    Special,
}

impl Default for CaseStatus {
    fn default() -> CaseStatus {
        CaseStatus::Common
    }
}

impl CaseStatus {
    /// Returns true if and only if this status indicates a case mapping that
    /// won't change the number of codepoints.
    pub fn is_fixed(&self) -> bool {
        *self != CaseStatus::Full
    }
}

impl FromStr for CaseStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<CaseStatus, Error> {
        match s {
            "C" => Ok(CaseStatus::Common),
            "F" => Ok(CaseStatus::Full),
            "S" => Ok(CaseStatus::Simple),
            "T" => Ok(CaseStatus::Special),
            _ => err!("unrecognized case status: '{}' \
                       (must be one of C, F, S or T)", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CaseFold, CaseStatus};

    #[test]
    fn parse_common() {
        let line = "0150; C; 0151; # LATIN CAPITAL LETTER O WITH DOUBLE ACUTE\n";
        let row: CaseFold = line.parse().unwrap();
        assert_eq!(row.codepoint, 0x0150);
        assert_eq!(row.status, CaseStatus::Common);
        assert_eq!(row.mapping, vec![0x0151]);
    }

    #[test]
    fn parse_full() {
        let line = "03B0; F; 03C5 0308 0301; # GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND TONOS\n";
        let row: CaseFold = line.parse().unwrap();
        assert_eq!(row.codepoint, 0x03B0);
        assert_eq!(row.status, CaseStatus::Full);
        assert_eq!(row.mapping, vec![0x03C5, 0x0308, 0x0301]);
    }

    #[test]
    fn parse_simple() {
        let line = "1F8F; S; 1F87; # GREEK CAPITAL LETTER ALPHA WITH DASIA AND PERISPOMENI AND PROSGEGRAMMENI\n";
        let row: CaseFold = line.parse().unwrap();
        assert_eq!(row.codepoint, 0x1F8F);
        assert_eq!(row.status, CaseStatus::Simple);
        assert_eq!(row.mapping, vec![0x1F87]);
    }

    #[test]
    fn parse_special() {
        let line = "0049; T; 0131; # LATIN CAPITAL LETTER I\n";
        let row: CaseFold = line.parse().unwrap();
        assert_eq!(row.codepoint, 0x0049);
        assert_eq!(row.status, CaseStatus::Special);
        assert_eq!(row.mapping, vec![0x0131]);
    }
}
