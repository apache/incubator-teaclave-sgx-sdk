use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_codepoint_association,
};
use error::Error;

/// A single row in the `DerivedCoreProperties.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoreProperty {
    /// The codepoint or codepoint range for this entry.
    pub codepoints: Codepoints,
    /// The property name assigned to the codepoints in this entry.
    pub property: String,
}

impl UcdFile for CoreProperty {
    fn relative_file_path() -> &'static Path {
        Path::new("DerivedCoreProperties.txt")
    }
}

impl UcdFileByCodepoint for CoreProperty {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl FromStr for CoreProperty {
    type Err = Error;

    fn from_str(line: &str) -> Result<CoreProperty, Error> {
        let (codepoints, property) = parse_codepoint_association(line)?;
        Ok(CoreProperty {
            codepoints: codepoints,
            property: property.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CoreProperty;

    #[test]
    fn parse_single() {
        let line = "1163D         ; Case_Ignorable # Mn       MODI SIGN ANUSVARA\n";
        let row: CoreProperty = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x1163D);
        assert_eq!(row.property, "Case_Ignorable");
    }

    #[test]
    fn parse_range() {
        let line = "11133..11134  ; Grapheme_Link # Mn   [2] CHAKMA VIRAMA..CHAKMA MAAYYAA\n";
        let row: CoreProperty = line.parse().unwrap();
        assert_eq!(row.codepoints, (0x11133, 0x11134));
        assert_eq!(row.property, "Grapheme_Link");
    }
}
