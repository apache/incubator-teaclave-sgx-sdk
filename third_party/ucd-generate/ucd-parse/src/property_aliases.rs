use std::path::Path;
use std::str::FromStr;

use regex::Regex;

use common::UcdFile;
use error::Error;

/// A single row in the `PropertyAliases.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PropertyAlias {
    /// An abbreviation for this property.
    pub abbreviation: String,
    /// The "long" name of this property.
    pub long: String,
    /// Additional aliases (if present).
    pub aliases: Vec<String>,
}

impl UcdFile for PropertyAlias {
    fn relative_file_path() -> &'static Path {
        Path::new("PropertyAliases.txt")
    }
}

impl FromStr for PropertyAlias {
    type Err = Error;

    fn from_str(line: &str) -> Result<PropertyAlias, Error> {
        lazy_static! {
            static ref PARTS: Regex = Regex::new(
                r"(?x)
                ^
                \s*(?P<abbrev>[^\s;]+)\s*;
                \s*(?P<long>[^\s;]+)\s*
                (?:;(?P<aliases>.*))?
                "
            ).unwrap();
            static ref ALIASES: Regex = Regex::new(
                r"\s*(?P<alias>[^\s;]+)\s*;?\s*"
            ).unwrap();
        };

        let caps = match PARTS.captures(line.trim()) {
            Some(caps) => caps,
            None => return err!("invalid PropertyAliases line: '{}'", line),
        };
        let mut aliases = vec![];
        if let Some(m) = caps.name("aliases") {
            for acaps in ALIASES.captures_iter(m.as_str()) {
                let alias = acaps.name("alias").unwrap().as_str();
                aliases.push(alias.to_string());
            }
        }
        Ok(PropertyAlias {
            abbreviation: caps.name("abbrev").unwrap().as_str().to_string(),
            long: caps.name("long").unwrap().as_str().to_string(),
            aliases: aliases,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PropertyAlias;

    #[test]
    fn parse1() {
        let line = "cjkAccountingNumeric     ; kAccountingNumeric\n";
        let row: PropertyAlias = line.parse().unwrap();
        assert_eq!(row.abbreviation, "cjkAccountingNumeric");
        assert_eq!(row.long, "kAccountingNumeric");
        assert!(row.aliases.is_empty());
    }

    #[test]
    fn parse2() {
        let line = "nv                       ; Numeric_Value\n";
        let row: PropertyAlias = line.parse().unwrap();
        assert_eq!(row.abbreviation, "nv");
        assert_eq!(row.long, "Numeric_Value");
        assert!(row.aliases.is_empty());
    }

    #[test]
    fn parse3() {
        let line = "scf                      ; Simple_Case_Folding         ; sfc\n";
        let row: PropertyAlias = line.parse().unwrap();
        assert_eq!(row.abbreviation, "scf");
        assert_eq!(row.long, "Simple_Case_Folding");
        assert_eq!(row.aliases, vec!["sfc"]);
    }

    #[test]
    fn parse4() {
        let line = "cjkRSUnicode             ; kRSUnicode                  ; Unicode_Radical_Stroke; URS\n";
        let row: PropertyAlias = line.parse().unwrap();
        assert_eq!(row.abbreviation, "cjkRSUnicode");
        assert_eq!(row.long, "kRSUnicode");
        assert_eq!(row.aliases, vec!["Unicode_Radical_Stroke", "URS"]);
    }

    #[test]
    fn parse5() {
        let line = "isc                      ; ISO_Comment";
        let row: PropertyAlias = line.parse().unwrap();
        assert_eq!(row.abbreviation, "isc");
        assert_eq!(row.long, "ISO_Comment");
        assert!(row.aliases.is_empty());
    }
}
