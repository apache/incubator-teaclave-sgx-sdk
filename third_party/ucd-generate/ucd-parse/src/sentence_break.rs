use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_break_test, parse_codepoint_association,
};
use error::Error;

/// A single row in the `auxiliary/SentenceBreakProperty.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SentenceBreak {
    /// The codepoint or codepoint range for this entry.
    pub codepoints: Codepoints,
    /// The property value assigned to the codepoints in this entry.
    pub value: String,
}

impl UcdFile for SentenceBreak {
    fn relative_file_path() -> &'static Path {
        Path::new("auxiliary/SentenceBreakProperty.txt")
    }
}

impl UcdFileByCodepoint for SentenceBreak {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl FromStr for SentenceBreak {
    type Err = Error;

    fn from_str(line: &str) -> Result<SentenceBreak, Error> {
        let (codepoints, value) = parse_codepoint_association(line)?;
        Ok(SentenceBreak {
            codepoints: codepoints,
            value: value.to_string(),
        })
    }
}

/// A single row in the `auxiliary/SentenceBreakTest.txt` file.
///
/// This file defines tests for the sentence break algorithm.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SentenceBreakTest {
    /// Each string is a UTF-8 encoded group of codepoints that make up a
    /// single sentence.
    pub sentences: Vec<String>,
    /// A human readable description of this test.
    pub comment: String,
}

impl UcdFile for SentenceBreakTest {
    fn relative_file_path() -> &'static Path {
        Path::new("auxiliary/SentenceBreakTest.txt")
    }
}

impl FromStr for SentenceBreakTest {
    type Err = Error;

    fn from_str(line: &str) -> Result<SentenceBreakTest, Error> {
        let (groups, comment) = parse_break_test(line)?;
        Ok(SentenceBreakTest {
            sentences: groups,
            comment: comment,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{SentenceBreak, SentenceBreakTest};

    #[test]
    fn parse_single() {
        let line = "11445         ; Extend # Mc       NEWA SIGN VISARGA\n";
        let row: SentenceBreak = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x11445);
        assert_eq!(row.value, "Extend");
    }

    #[test]
    fn parse_range() {
        let line = "FE31..FE32    ; SContinue # Pd   [2] PRESENTATION FORM FOR VERTICAL EM DASH..PRESENTATION FORM FOR VERTICAL EN DASH\n";
        let row: SentenceBreak = line.parse().unwrap();
        assert_eq!(row.codepoints, (0xFE31, 0xFE32));
        assert_eq!(row.value, "SContinue");
    }

    #[test]
    fn parse_test() {
        let line = "÷ 2060 × 5B57 × 2060 × 002E × 2060 ÷ 5B57 × 2060 × 2060 ÷	#  ÷ [0.2] WORD JOINER (Format_FE) × [998.0] CJK UNIFIED IDEOGRAPH-5B57 (OLetter) × [5.0] WORD JOINER (Format_FE) × [998.0] FULL STOP (ATerm) × [5.0] WORD JOINER (Format_FE) ÷ [11.0] CJK UNIFIED IDEOGRAPH-5B57 (OLetter) × [5.0] WORD JOINER (Format_FE) × [5.0] WORD JOINER (Format_FE) ÷ [0.3]";

        let row: SentenceBreakTest = line.parse().unwrap();
        assert_eq!(row.sentences, vec![
            "\u{2060}\u{5B57}\u{2060}\u{002E}\u{2060}",
            "\u{5B57}\u{2060}\u{2060}",
        ]);
        assert!(row.comment.contains("[5.0] WORD JOINER (Format_FE)"));
    }
}
