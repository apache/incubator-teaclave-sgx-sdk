
use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_codepoint_association,
};
use error::Error;

/// A single row in the `DerivedAge.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Age {
    /// The codepoint or codepoint range for this entry.
    pub codepoints: Codepoints,
    /// The age assigned to the codepoints in this entry.
    pub age: String,
}

impl UcdFile for Age {
    fn relative_file_path() -> &'static Path {
        Path::new("DerivedAge.txt")
    }
}

impl UcdFileByCodepoint for Age {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl FromStr for Age {
    type Err = Error;

    fn from_str(line: &str) -> Result<Age, Error> {
        let (codepoints, script) = parse_codepoint_association(line)?;
        Ok(Age {
            codepoints: codepoints,
            age: script.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Age;

    #[test]
    fn parse_single() {
        let line = "2BD2          ; 10.0 #       GROUP MARK\n";
        let row: Age = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x2BD2);
        assert_eq!(row.age, "10.0");
    }

    #[test]
    fn parse_range() {
        let line = "11D0B..11D36  ; 10.0 #  [44] MASARAM GONDI LETTER AU..MASARAM GONDI VOWEL SIGN VOCALIC R\n";
        let row: Age = line.parse().unwrap();
        assert_eq!(row.codepoints, (0x11D0B, 0x11D36));
        assert_eq!(row.age, "10.0");
    }
}
