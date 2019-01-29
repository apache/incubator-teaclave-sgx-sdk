use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_codepoint_association,
};
use error::Error;

/// A single row in the `Scripts.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Script {
    /// The codepoint or codepoint range for this entry.
    pub codepoints: Codepoints,
    /// The script name assigned to the codepoints in this entry.
    pub script: String,
}

impl UcdFile for Script {
    fn relative_file_path() -> &'static Path {
        Path::new("Scripts.txt")
    }
}

impl UcdFileByCodepoint for Script {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl FromStr for Script {
    type Err = Error;

    fn from_str(line: &str) -> Result<Script, Error> {
        let (codepoints, script) = parse_codepoint_association(line)?;
        Ok(Script {
            codepoints: codepoints,
            script: script.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Script;

    #[test]
    fn parse_single() {
        let line = "10A7F         ; Old_South_Arabian # Po       OLD SOUTH ARABIAN NUMERIC INDICATOR\n";
        let row: Script = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x10A7F);
        assert_eq!(row.script, "Old_South_Arabian");
    }

    #[test]
    fn parse_range() {
        let line = "1200..1248    ; Ethiopic # Lo  [73] ETHIOPIC SYLLABLE HA..ETHIOPIC SYLLABLE QWA\n";
        let row: Script = line.parse().unwrap();
        assert_eq!(row.codepoints, (0x1200, 0x1248));
        assert_eq!(row.script, "Ethiopic");
    }
}
