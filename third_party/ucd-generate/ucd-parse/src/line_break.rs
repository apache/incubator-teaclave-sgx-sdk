use std::path::Path;
use std::str::FromStr;

use common::{UcdFile, parse_break_test};
use error::Error;

/// A single row in the `auxiliary/LineBreakTest.txt` file.
///
/// This file defines tests for the line break algorithm.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LineBreakTest {
    /// Each string is a UTF-8 encoded group of codepoints that make up a
    /// single line.
    pub lines: Vec<String>,
    /// A human readable description of this test.
    pub comment: String,
}

impl UcdFile for LineBreakTest {
    fn relative_file_path() -> &'static Path {
        Path::new("auxiliary/LineBreakTest.txt")
    }
}

impl FromStr for LineBreakTest {
    type Err = Error;

    fn from_str(line: &str) -> Result<LineBreakTest, Error> {
        let (groups, comment) = parse_break_test(line)?;
        Ok(LineBreakTest {
            lines: groups,
            comment: comment,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::LineBreakTest;

    #[test]
    fn parse_test() {
        let line = "× 1F1F7 × 1F1FA ÷ 1F1F8 × 1F1EA ÷   #  × [0.3] REGIONAL INDICATOR SYMBOL LETTER R (RI) × [30.11] REGIONAL INDICATOR SYMBOL LETTER U (RI) ÷ [30.13] REGIONAL INDICATOR SYMBOL LETTER S (RI) × [30.11] REGIONAL INDICATOR SYMBOL LETTER E (RI) ÷ [0.3]";

        let row: LineBreakTest = line.parse().unwrap();
        assert_eq!(row.lines, vec![
            "\u{1F1F7}\u{1F1FA}",
            "\u{1F1F8}\u{1F1EA}",
        ]);
        assert!(row.comment.ends_with("(RI) ÷ [0.3]"));
    }
}
