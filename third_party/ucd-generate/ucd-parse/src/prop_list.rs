use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_codepoint_association,
};
use error::Error;

/// A single row in the `PropList.txt` file.
///
/// The `PropList.txt` file is the source of truth on several Unicode
/// properties.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Property {
    /// The codepoint or codepoint range for this entry.
    pub codepoints: Codepoints,
    /// The property name assigned to the codepoints in this entry.
    pub property: String,
}

impl UcdFile for Property {
    fn relative_file_path() -> &'static Path {
        Path::new("PropList.txt")
    }
}

impl UcdFileByCodepoint for Property {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl FromStr for Property {
    type Err = Error;

    fn from_str(line: &str) -> Result<Property, Error> {
        let (codepoints, property) = parse_codepoint_association(line)?;
        Ok(Property {
            codepoints: codepoints,
            property: property.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Property;

    #[test]
    fn parse_single() {
        let line = "061C          ; Bidi_Control # Cf       ARABIC LETTER MARK\n";
        let row: Property = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x061C);
        assert_eq!(row.property, "Bidi_Control");
    }

    #[test]
    fn parse_range() {
        let line = "0009..000D    ; White_Space # Cc   [5] <control-0009>..<control-000D>\n";
        let row: Property = line.parse().unwrap();
        assert_eq!(row.codepoints, (0x0009, 0x000D));
        assert_eq!(row.property, "White_Space");
    }
}
