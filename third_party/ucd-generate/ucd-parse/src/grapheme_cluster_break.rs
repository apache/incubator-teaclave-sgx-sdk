use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_break_test, parse_codepoint_association,
};
use error::Error;

/// A single row in the `auxiliary/GraphemeBreakProperty.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphemeClusterBreak {
    /// The codepoint or codepoint range for this entry.
    pub codepoints: Codepoints,
    /// The property value assigned to the codepoints in this entry.
    pub value: String,
}

impl UcdFile for GraphemeClusterBreak {
    fn relative_file_path() -> &'static Path {
        Path::new("auxiliary/GraphemeBreakProperty.txt")
    }
}

impl UcdFileByCodepoint for GraphemeClusterBreak {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl FromStr for GraphemeClusterBreak {
    type Err = Error;

    fn from_str(line: &str) -> Result<GraphemeClusterBreak, Error> {
        let (codepoints, value) = parse_codepoint_association(line)?;
        Ok(GraphemeClusterBreak {
            codepoints: codepoints,
            value: value.to_string(),
        })
    }
}

/// A single row in the `auxiliary/GraphemeBreakTest.txt` file.
///
/// This file defines tests for the grapheme cluster break algorithm.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphemeClusterBreakTest {
    /// Each string is a UTF-8 encoded group of codepoints that make up a
    /// single grapheme cluster.
    pub grapheme_clusters: Vec<String>,
    /// A human readable description of this test.
    pub comment: String,
}

impl UcdFile for GraphemeClusterBreakTest {
    fn relative_file_path() -> &'static Path {
        Path::new("auxiliary/GraphemeBreakTest.txt")
    }
}

impl FromStr for GraphemeClusterBreakTest {
    type Err = Error;

    fn from_str(line: &str) -> Result<GraphemeClusterBreakTest, Error> {
        let (groups, comment) = parse_break_test(line)?;
        Ok(GraphemeClusterBreakTest {
            grapheme_clusters: groups,
            comment: comment,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{GraphemeClusterBreak, GraphemeClusterBreakTest};

    #[test]
    fn parse_single() {
        let line = "093B          ; SpacingMark # Mc       DEVANAGARI VOWEL SIGN OOE\n";
        let row: GraphemeClusterBreak = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x093B);
        assert_eq!(row.value, "SpacingMark");
    }

    #[test]
    fn parse_range() {
        let line = "1F1E6..1F1FF  ; Regional_Indicator # So  [26] REGIONAL INDICATOR SYMBOL LETTER A..REGIONAL INDICATOR SYMBOL LETTER Z\n";
        let row: GraphemeClusterBreak = line.parse().unwrap();
        assert_eq!(row.codepoints, (0x1F1E6, 0x1F1FF));
        assert_eq!(row.value, "Regional_Indicator");
    }

    #[test]
    fn parse_test() {
        let line = "÷ 0061 × 1F3FF ÷ 1F476 × 200D × 1F6D1 ÷	#  ÷ [0.2] LATIN SMALL LETTER A (Other) × [9.0] EMOJI MODIFIER FITZPATRICK TYPE-6 (Extend) ÷ [999.0] BABY (ExtPict) × [9.0] ZERO WIDTH JOINER (ZWJ_ExtCccZwj) × [11.0] OCTAGONAL SIGN (ExtPict) ÷ [0.3]\n";

        let row: GraphemeClusterBreakTest = line.parse().unwrap();
        assert_eq!(row.grapheme_clusters, vec![
            "\u{0061}\u{1F3FF}",
            "\u{1F476}\u{200D}\u{1F6D1}",
        ]);
        assert!(row.comment.starts_with("÷ [0.2] LATIN SMALL LETTER A"));
    }
}
