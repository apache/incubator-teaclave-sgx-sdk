use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_codepoint_association,
};
use error::Error;

/// A single row in the `emoji-data.txt` file.
///
/// The `emoji-data.txt` file is the source of truth on several Emoji-related
/// Unicode properties.
///
/// Note that `emoji-data.txt` is not formally part of the Unicode Character
/// Database. You can download the Emoji data files separately here:
/// https://unicode.org/Public/emoji/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EmojiProperty {
    /// The codepoint or codepoint range for this entry.
    pub codepoints: Codepoints,
    /// The property name assigned to the codepoints in this entry.
    pub property: String,
}

impl UcdFile for EmojiProperty {
    fn relative_file_path() -> &'static Path {
        Path::new("emoji-data.txt")
    }
}

impl UcdFileByCodepoint for EmojiProperty {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl FromStr for EmojiProperty {
    type Err = Error;

    fn from_str(line: &str) -> Result<EmojiProperty, Error> {
        let (codepoints, property) = parse_codepoint_association(line)?;
        Ok(EmojiProperty {
            codepoints: codepoints,
            property: property.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::EmojiProperty;

    #[test]
    fn parse_single() {
        let line = "24C2          ; Emoji                #  1.1  [1] (Ⓜ️)       circled M\n";
        let row: EmojiProperty = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x24C2);
        assert_eq!(row.property, "Emoji");
    }

    #[test]
    fn parse_range() {
        let line = "1FA6E..1FFFD  ; Extended_Pictographic#   NA[1424] (🩮️..🿽️)   <reserved-1FA6E>..<reserved-1FFFD>\n";
        let row: EmojiProperty = line.parse().unwrap();
        assert_eq!(row.codepoints, (0x1FA6E, 0x1FFFD));
        assert_eq!(row.property, "Extended_Pictographic");
    }
}
