use std::path::Path;
use std::str::FromStr;

use regex::Regex;

use common::UcdFile;
use error::Error;

/// A single row in the `PropertyValueAliases.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PropertyValueAlias {
    /// The property name for which this value alias applies.
    pub property: String,
    /// A numeric abbreviation for this property value, if present. (This is
    /// seemingly only present for the `ccc`/`Canonical_Combining_Class`
    /// property.)
    pub numeric: Option<u8>,
    /// An abbreviation for this property value.
    pub abbreviation: String,
    /// The "long" form of this property value.
    pub long: String,
    /// Additional value aliases (if present).
    pub aliases: Vec<String>,
}

impl UcdFile for PropertyValueAlias {
    fn relative_file_path() -> &'static Path {
        Path::new("PropertyValueAliases.txt")
    }
}

impl FromStr for PropertyValueAlias {
    type Err = Error;

    fn from_str(line: &str) -> Result<PropertyValueAlias, Error> {
        lazy_static! {
            static ref PARTS: Regex = Regex::new(
                r"(?x)
                ^
                \s*(?P<prop>[^\s;]+)\s*;
                \s*(?P<abbrev>[^\s;]+)\s*;
                \s*(?P<long>[^\s;]+)\s*
                (?:;(?P<aliases>.*))?
                "
            ).unwrap();
            static ref PARTS_CCC: Regex = Regex::new(
                r"(?x)
                ^
                ccc;
                \s*(?P<num_class>[0-9]+)\s*;
                \s*(?P<abbrev>[^\s;]+)\s*;
                \s*(?P<long>[^\s;]+)
                "
            ).unwrap();
            static ref ALIASES: Regex = Regex::new(
                r"\s*(?P<alias>[^\s;]+)\s*;?\s*"
            ).unwrap();
        };

        if line.starts_with("ccc;") {
            let caps = match PARTS_CCC.captures(line.trim()) {
                Some(caps) => caps,
                None => return err!("invalid PropertyValueAliases (ccc) line"),
            };
            let n = match caps["num_class"].parse() {
                Ok(n) => n,
                Err(err) => return err!(
                    "failed to parse ccc number '{}': {}",
                    &caps["num_class"], err),
            };
            let abbrev = caps.name("abbrev").unwrap().as_str();
            let long = caps.name("long").unwrap().as_str();
            return Ok(PropertyValueAlias {
                property: line[0..3].to_string(),
                numeric: Some(n),
                abbreviation: abbrev.to_string(),
                long: long.to_string(),
                aliases: vec![],
            });
        }

        let caps = match PARTS.captures(line.trim()) {
            Some(caps) => caps,
            None => return err!("invalid PropertyValueAliases line"),
        };
        let mut aliases = vec![];
        if let Some(m) = caps.name("aliases") {
            for acaps in ALIASES.captures_iter(m.as_str()) {
                let alias = acaps.name("alias").unwrap().as_str();
                if alias == "#" {
                    // This starts a comment, so stop reading.
                    break;
                }
                aliases.push(alias.to_string());
            }
        }
        Ok(PropertyValueAlias {
            property: caps.name("prop").unwrap().as_str().to_string(),
            numeric: None,
            abbreviation: caps.name("abbrev").unwrap().as_str().to_string(),
            long: caps.name("long").unwrap().as_str().to_string(),
            aliases: aliases,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PropertyValueAlias;

    #[test]
    fn parse1() {
        let line = "blk; Arabic_PF_A                      ; Arabic_Presentation_Forms_A      ; Arabic_Presentation_Forms-A\n";
        let row: PropertyValueAlias = line.parse().unwrap();
        assert_eq!(row.property, "blk");
        assert_eq!(row.numeric, None);
        assert_eq!(row.abbreviation, "Arabic_PF_A");
        assert_eq!(row.long, "Arabic_Presentation_Forms_A");
        assert_eq!(row.aliases, vec!["Arabic_Presentation_Forms-A"]);
    }

    #[test]
    fn parse2() {
        let line = "AHex; N                               ; No                               ; F                                ; False\n";
        let row: PropertyValueAlias = line.parse().unwrap();
        assert_eq!(row.property, "AHex");
        assert_eq!(row.numeric, None);
        assert_eq!(row.abbreviation, "N");
        assert_eq!(row.long, "No");
        assert_eq!(row.aliases, vec!["F", "False"]);
    }

    #[test]
    fn parse3() {
        let line = "age; 1.1                              ; V1_1\n";
        let row: PropertyValueAlias = line.parse().unwrap();
        assert_eq!(row.property, "age");
        assert_eq!(row.numeric, None);
        assert_eq!(row.abbreviation, "1.1");
        assert_eq!(row.long, "V1_1");
        assert!(row.aliases.is_empty());
    }

    #[test]
    fn parse4() {
        let line = "ccc;   0; NR                         ; Not_Reordered\n";
        let row: PropertyValueAlias = line.parse().unwrap();
        assert_eq!(row.property, "ccc");
        assert_eq!(row.numeric, Some(0));
        assert_eq!(row.abbreviation, "NR");
        assert_eq!(row.long, "Not_Reordered");
        assert!(row.aliases.is_empty());
    }

    #[test]
    fn parse5() {
        let line = "ccc; 133; CCC133                     ; CCC133 # RESERVED\n";
        let row: PropertyValueAlias = line.parse().unwrap();
        assert_eq!(row.property, "ccc");
        assert_eq!(row.numeric, Some(133));
        assert_eq!(row.abbreviation, "CCC133");
        assert_eq!(row.long, "CCC133");
        assert!(row.aliases.is_empty());
    }

    #[test]
    fn parse6() {
        let line = "gc ; P                                ; Punctuation                      ; punct                            # Pc | Pd | Pe | Pf | Pi | Po | Ps\n";
        let row: PropertyValueAlias = line.parse().unwrap();
        assert_eq!(row.property, "gc");
        assert_eq!(row.numeric, None);
        assert_eq!(row.abbreviation, "P");
        assert_eq!(row.long, "Punctuation");
        assert_eq!(row.aliases, vec!["punct"]);
    }
}
