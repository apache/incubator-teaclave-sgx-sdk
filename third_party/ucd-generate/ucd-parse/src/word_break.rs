use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_break_test, parse_codepoint_association,
};
use error::Error;

/// A single row in the `auxiliary/WordBreakProperty.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WordBreak {
    /// The codepoint or codepoint range for this entry.
    pub codepoints: Codepoints,
    /// The property value assigned to the codepoints in this entry.
    pub value: String,
}

impl UcdFile for WordBreak {
    fn relative_file_path() -> &'static Path {
        Path::new("auxiliary/WordBreakProperty.txt")
    }
}

impl UcdFileByCodepoint for WordBreak {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl FromStr for WordBreak {
    type Err = Error;

    fn from_str(line: &str) -> Result<WordBreak, Error> {
        let (codepoints, value) = parse_codepoint_association(line)?;
        Ok(WordBreak {
            codepoints: codepoints,
            value: value.to_string(),
        })
    }
}

/// A single row in the `auxiliary/WordBreakTest.txt` file.
///
/// This file defines tests for the word break algorithm.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WordBreakTest {
    /// Each string is a UTF-8 encoded group of codepoints that make up a
    /// single word.
    pub words: Vec<String>,
    /// A human readable description of this test.
    pub comment: String,
}

impl UcdFile for WordBreakTest {
    fn relative_file_path() -> &'static Path {
        Path::new("auxiliary/WordBreakTest.txt")
    }
}

impl FromStr for WordBreakTest {
    type Err = Error;

    fn from_str(line: &str) -> Result<WordBreakTest, Error> {
        let (groups, comment) = parse_break_test(line)?;
        Ok(WordBreakTest {
            words: groups,
            comment: comment,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{WordBreak, WordBreakTest};

    #[test]
    fn parse_single() {
        let line = "0A83          ; Extend # Mc       GUJARATI SIGN VISARGA\n";
        let row: WordBreak = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x0A83);
        assert_eq!(row.value, "Extend");
    }

    #[test]
    fn parse_range() {
        let line = "104A0..104A9  ; Numeric # Nd  [10] OSMANYA DIGIT ZERO..OSMANYA DIGIT NINE\n";
        let row: WordBreak = line.parse().unwrap();
        assert_eq!(row.codepoints, (0x104A0, 0x104A9));
        assert_eq!(row.value, "Numeric");
    }

    #[test]
    fn parse_test() {
        let line = "÷ 0031 ÷ 0027 × 0308 ÷ 0061 ÷ 0027 × 2060 ÷	#  ÷ [0.2] DIGIT ONE (Numeric) ÷ [999.0] APOSTROPHE (Single_Quote) × [4.0] COMBINING DIAERESIS (Extend_FE) ÷ [999.0] LATIN SMALL LETTER A (ALetter) ÷ [999.0] APOSTROPHE (Single_Quote) × [4.0] WORD JOINER (Format_FE) ÷ [0.3]";

        let row: WordBreakTest = line.parse().unwrap();
        assert_eq!(row.words, vec![
            "\u{0031}",
            "\u{0027}\u{0308}",
            "\u{0061}",
            "\u{0027}\u{2060}",
        ]);
        assert!(row.comment.contains("[4.0] COMBINING DIAERESIS (Extend_FE)"));
    }
}
