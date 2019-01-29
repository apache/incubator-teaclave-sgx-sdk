use std::path::Path;
use std::str::FromStr;

use regex::Regex;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoint, CodepointIter,
    parse_codepoint_sequence,
};
use error::Error;

/// A single row in the `SpecialCasing.txt` file.
///
/// Note that a single codepoint may be mapped multiple times. In particular,
/// a single codepoint might have mappings based on distinct language sensitive
/// conditions (e.g., `U+0307`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SpecialCaseMapping {
    /// The codepoint that is being mapped.
    pub codepoint: Codepoint,
    /// The lowercase mapping, which may be empty.
    pub lowercase: Vec<Codepoint>,
    /// The titlecase mapping, which may be empty.
    pub titlecase: Vec<Codepoint>,
    /// The uppercase mapping, which may be empty.
    pub uppercase: Vec<Codepoint>,
    /// A list of language specific conditions, see `SpecialCasing.txt` for
    /// more details.
    pub conditions: Vec<String>,
}

impl UcdFile for SpecialCaseMapping {
    fn relative_file_path() -> &'static Path {
        Path::new("SpecialCasing.txt")
    }
}

impl UcdFileByCodepoint for SpecialCaseMapping {
    fn codepoints(&self) -> CodepointIter {
        self.codepoint.into_iter()
    }
}

impl FromStr for SpecialCaseMapping {
    type Err = Error;

    fn from_str(line: &str) -> Result<SpecialCaseMapping, Error> {
        lazy_static! {
            static ref PARTS: Regex = Regex::new(
                r"(?x)
                ^
                \s*(?P<codepoint>[^\s;]+)\s*;
                \s*(?P<lower>[^;]+)\s*;
                \s*(?P<title>[^;]+)\s*;
                \s*(?P<upper>[^;]+)\s*;
                \s*(?P<conditions>[^;\x23]+)?
                "
            ).unwrap();
        };

        let caps = match PARTS.captures(line.trim()) {
            Some(caps) => caps,
            None => return err!("invalid SpecialCasing line: '{}'", line),
        };
        let conditions = caps
            .name("conditions")
            .map(|x| x.as_str().trim().split_whitespace().map(|c| c.to_string()).collect())
            .unwrap_or(vec![]);
        Ok(SpecialCaseMapping {
            codepoint: caps["codepoint"].parse()?,
            lowercase: parse_codepoint_sequence(&caps["lower"])?,
            titlecase: parse_codepoint_sequence(&caps["title"])?,
            uppercase: parse_codepoint_sequence(&caps["upper"])?,
            conditions: conditions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SpecialCaseMapping;

    #[test]
    fn parse_no_conds() {
        let line = "1F52; 1F52; 03A5 0313 0300; 03A5 0313 0300; # GREEK SMALL LETTER UPSILON WITH PSILI AND VARIA\n";
        let row: SpecialCaseMapping = line.parse().unwrap();
        assert_eq!(row.codepoint, 0x1F52);
        assert_eq!(row.lowercase, vec![0x1F52]);
        assert_eq!(row.titlecase, vec![0x03A5, 0x0313, 0x0300]);
        assert_eq!(row.uppercase, vec![0x03A5, 0x0313, 0x0300]);
        assert!(row.conditions.is_empty());
    }

    #[test]
    fn parse_conds()  {
        let line = "0307; ; 0307; 0307; tr After_I; # COMBINING DOT ABOVE\n";
        let row: SpecialCaseMapping = line.parse().unwrap();
        assert_eq!(row.codepoint, 0x0307);
        assert!(row.lowercase.is_empty());
        assert_eq!(row.titlecase, vec![0x0307]);
        assert_eq!(row.uppercase, vec![0x0307]);
        assert_eq!(row.conditions, vec!["tr", "After_I"]);
    }
}
