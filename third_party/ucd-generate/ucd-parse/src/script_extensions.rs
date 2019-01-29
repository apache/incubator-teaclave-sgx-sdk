use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_codepoint_association,
};
use error::Error;

/// A single row in the `ScriptExtensions.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScriptExtension {
    /// The codepoint or codepoint range for this entry.
    pub codepoints: Codepoints,
    /// The script extension names assigned to the codepoints in this entry.
    pub scripts: Vec<String>,
}

impl UcdFile for ScriptExtension {
    fn relative_file_path() -> &'static Path {
        Path::new("ScriptExtensions.txt")
    }
}

impl UcdFileByCodepoint for ScriptExtension {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl FromStr for ScriptExtension {
    type Err = Error;

    fn from_str(line: &str) -> Result<ScriptExtension, Error> {
        let (codepoints, scripts) = parse_codepoint_association(line)?;
        Ok(ScriptExtension {
            codepoints: codepoints,
            scripts: scripts.split_whitespace().map(str::to_string).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ScriptExtension;

    #[test]
    fn parse_single() {
        let line = "060C          ; Arab Syrc Thaa # Po       ARABIC COMMA\n";
        let row: ScriptExtension = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x060C);
        assert_eq!(row.scripts, vec!["Arab", "Syrc", "Thaa"]);
    }

    #[test]
    fn parse_range() {
        let line = "A836..A837    ; Deva Gujr Guru Kthi Mahj Modi Sind Takr Tirh # So   [2] NORTH INDIC QUARTER MARK..NORTH INDIC PLACEHOLDER MARK\n";
        let row: ScriptExtension = line.parse().unwrap();
        assert_eq!(row.codepoints, (0xA836, 0xA837));
        assert_eq!(row.scripts, vec![
            "Deva", "Gujr", "Guru", "Kthi", "Mahj", "Modi", "Sind", "Takr",
            "Tirh",
        ]);
    }
}
