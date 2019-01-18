use std::fmt;
use std::iter;
use std::ops::Range;
use std::path::Path;
use std::str::FromStr;

use regex::Regex;

use common::{UcdFile, UcdFileByCodepoint, Codepoint, CodepointIter};
use error::Error;

/// Represents a single row in the `UnicodeData.txt` file.
///
/// These fields were taken from UAX44, Table 9, as part of the documentation
/// for the
/// [`UnicodeData.txt` file](http://www.unicode.org/reports/tr44/#UnicodeData.txt).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UnicodeData {
    /// The codepoint corresponding to this row.
    pub codepoint: Codepoint,
    /// The name of this codepoint.
    pub name: String,
    /// The "general category" of this codepoint.
    pub general_category: String,
    /// The class of this codepoint used in the Canonical Ordering Algorithm.
    ///
    /// Note that some classes map to a particular symbol. See
    /// [UAX44, Table 15](http://www.unicode.org/reports/tr44/#Canonical_Combining_Class_Values).
    pub canonical_combining_class: u8,
    /// The bidirectional class of this codepoint.
    ///
    /// Possible values are listed in
    /// [UAX44, Table 13](http://www.unicode.org/reports/tr44/#Bidi_Class_Values).
    pub bidi_class: String,
    /// The decomposition mapping for this codepoint. This includes its
    /// formatting tag (if present).
    pub decomposition: UnicodeDataDecomposition,
    /// A decimal numeric representation of this codepoint, if it has the
    /// property `Numeric_Type=Decimal`.
    pub numeric_type_decimal: Option<u8>,
    /// A decimal numeric representation of this codepoint, if it has the
    /// property `Numeric_Type=Digit`. Note that while this field is still
    /// populated for existing codepoints, no new codepoints will have this
    /// field populated.
    pub numeric_type_digit: Option<u8>,
    /// A decimal or rational numeric representation of this codepoint, if it
    /// has the property `Numeric_Type=Numeric`.
    pub numeric_type_numeric: Option<UnicodeDataNumeric>,
    /// A boolean indicating whether this codepoint is "mirrored" in
    /// bidirectional text.
    pub bidi_mirrored: bool,
    /// The "old" Unicode 1.0 or ISO 6429 name of this codepoint. Note that
    /// this field is empty unless it is significantly different from
    /// the `name` field.
    pub unicode1_name: String,
    /// The ISO 10464 comment field. This no longer contains any non-NULL
    /// values.
    pub iso_comment: String,
    /// This codepoint's simple uppercase mapping, if it exists.
    pub simple_uppercase_mapping: Option<Codepoint>,
    /// This codepoint's simple lowercase mapping, if it exists.
    pub simple_lowercase_mapping: Option<Codepoint>,
    /// This codepoint's simple titlecase mapping, if it exists.
    pub simple_titlecase_mapping: Option<Codepoint>,
}

impl UcdFile for UnicodeData {
    fn relative_file_path() -> &'static Path {
        Path::new("UnicodeData.txt")
    }
}

impl UcdFileByCodepoint for UnicodeData {
    fn codepoints(&self) -> CodepointIter {
        self.codepoint.into_iter()
    }
}

impl UnicodeData {
    /// Returns true if and only if this record corresponds to the start of a
    /// range.
    pub fn is_range_start(&self) -> bool {
        self.name.starts_with('<')
        && self.name.ends_with('>')
        && self.name.contains("First")
    }

    /// Returns true if and only if this record corresponds to the end of a
    /// range.
    pub fn is_range_end(&self) -> bool {
        self.name.starts_with('<')
        && self.name.ends_with('>')
        && self.name.contains("Last")
    }
}

impl FromStr for UnicodeData {
    type Err = Error;

    fn from_str(line: &str) -> Result<UnicodeData, Error> {
        lazy_static! {
            static ref PARTS: Regex = Regex::new(
                r"(?x)
                ^
                ([A-Z0-9]+);  #  1; codepoint
                ([^;]+);      #  2; name
                ([^;]+);      #  3; general category
                ([0-9]+);     #  4; canonical combining class
                ([^;]+);      #  5; bidi class
                ([^;]*);      #  6; decomposition
                ([0-9]*);     #  7; numeric type decimal
                ([0-9]*);     #  8; numeric type digit
                ([-0-9/]*);   #  9; numeric type numeric
                ([YN]);       # 10; bidi mirrored
                ([^;]*);      # 11; unicode1 name
                ([^;]*);      # 12; ISO comment
                ([^;]*);      # 13; simple uppercase mapping
                ([^;]*);      # 14; simple lowercase mapping
                ([^;]*)       # 15; simple titlecase mapping
                $
                "
            ).unwrap();
        };
        let caps = match PARTS.captures(line.trim()) {
            Some(caps) => caps,
            None => return err!("invalid UnicodeData line"),
        };
        let capget = |n| caps.get(n).unwrap().as_str();
        let mut data = UnicodeData::default();

        data.codepoint = capget(1).parse()?;
        data.name = capget(2).to_string();
        data.general_category = capget(3).to_string();
        data.canonical_combining_class = match capget(4).parse() {
            Ok(n) => n,
            Err(err) => return err!(
                "failed to parse canonical combining class '{}': {}",
                capget(4), err),
        };
        data.bidi_class = capget(5).to_string();
        if !caps[6].is_empty() {
            data.decomposition = caps[6].parse()?;
        } else {
            data.decomposition.push(data.codepoint)?;
        }
        if !capget(7).is_empty() {
            data.numeric_type_decimal = Some(match capget(7).parse() {
                Ok(n) => n,
                Err(err) => return err!(
                    "failed to parse numeric type decimal '{}': {}",
                    capget(7), err),
            });
        }
        if !capget(8).is_empty() {
            data.numeric_type_digit = Some(match capget(8).parse() {
                Ok(n) => n,
                Err(err) => return err!(
                    "failed to parse numeric type digit '{}': {}",
                    capget(8), err),
            });
        }
        if !capget(9).is_empty() {
            data.numeric_type_numeric = Some(capget(9).parse()?);
        }
        data.bidi_mirrored = capget(10) == "Y";
        data.unicode1_name = capget(11).to_string();
        data.iso_comment = capget(12).to_string();
        if !capget(13).is_empty() {
            data.simple_uppercase_mapping = Some(capget(13).parse()?);
        }
        if !capget(14).is_empty() {
            data.simple_lowercase_mapping = Some(capget(14).parse()?);
        }
        if !capget(15).is_empty() {
            data.simple_titlecase_mapping = Some(capget(15).parse()?);
        }
        Ok(data)
    }
}

impl fmt::Display for UnicodeData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{};", self.codepoint)?;
        write!(f, "{};", self.name)?;
        write!(f, "{};", self.general_category)?;
        write!(f, "{};", self.canonical_combining_class)?;
        write!(f, "{};", self.bidi_class)?;
        if self.decomposition.is_canonical()
            && self.decomposition.mapping() == &[self.codepoint]
        {
            write!(f, ";")?;
        } else {
            write!(f, "{};", self.decomposition)?;
        }
        if let Some(n) = self.numeric_type_decimal {
            write!(f, "{};", n)?;
        } else {
            write!(f, ";")?;
        }
        if let Some(n) = self.numeric_type_digit {
            write!(f, "{};", n)?;
        } else {
            write!(f, ";")?;
        }
        if let Some(n) = self.numeric_type_numeric {
            write!(f, "{};", n)?;
        } else {
            write!(f, ";")?;
        }
        write!(f, "{};", if self.bidi_mirrored { "Y" } else { "N" })?;
        write!(f, "{};", self.unicode1_name)?;
        write!(f, "{};", self.iso_comment)?;
        if let Some(cp) = self.simple_uppercase_mapping {
            write!(f, "{};", cp)?;
        } else {
            write!(f, ";")?;
        }
        if let Some(cp) = self.simple_lowercase_mapping {
            write!(f, "{};", cp)?;
        } else {
            write!(f, ";")?;
        }
        if let Some(cp) = self.simple_titlecase_mapping {
            write!(f, "{}", cp)?;
        }
        Ok(())
    }
}

/// Represents a decomposition mapping of a single row in the
/// `UnicodeData.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UnicodeDataDecomposition {
    /// The formatting tag associated with this mapping, if present.
    pub tag: Option<UnicodeDataDecompositionTag>,
    /// The number of codepoints in this mapping.
    pub len: usize,
    /// The codepoints in the mapping. Entries beyond `len` in the mapping
    /// are always U+0000. If no mapping was present, then this always contains
    /// a single codepoint corresponding to this row's character.
    pub mapping: [Codepoint; 18],
}

impl UnicodeDataDecomposition {
    /// Create a new decomposition mapping with the given tag and codepoints.
    ///
    /// If there are too many codepoints, then an error is returned.
    pub fn new(
        tag: Option<UnicodeDataDecompositionTag>,
        mapping: &[Codepoint],
    ) -> Result<UnicodeDataDecomposition, Error> {
        let mut x = UnicodeDataDecomposition::default();
        x.tag = tag;
        for &cp in mapping {
            x.push(cp)?;
        }
        Ok(x)
    }

    /// Add a new codepoint to this decomposition's mapping.
    ///
    /// If the mapping is already full, then this returns an error.
    pub fn push(&mut self, cp: Codepoint) -> Result<(), Error> {
        if self.len >= self.mapping.len() {
            return err!("invalid decomposition mapping (too many codepoints)");
        }
        self.mapping[self.len] = cp;
        self.len += 1;
        Ok(())
    }

    /// Return the mapping as a slice of codepoints. The slice returned
    /// has length equivalent to the number of codepoints in this mapping.
    pub fn mapping(&self) -> &[Codepoint] {
        &self.mapping[..self.len]
    }

    /// Returns true if and only if this decomposition mapping is canonical.
    pub fn is_canonical(&self) -> bool {
        self.tag.is_none()
    }
}

impl FromStr for UnicodeDataDecomposition {
    type Err = Error;

    fn from_str(s: &str) -> Result<UnicodeDataDecomposition, Error> {
        lazy_static! {
            static ref WITH_TAG: Regex = Regex::new(
                r"^(?:<(?P<tag>[^>]+)>)?\s*(?P<chars>[\s0-9A-F]+)$"
            ).unwrap();
            static ref CHARS: Regex = Regex::new(r"[0-9A-F]+").unwrap();
        };
        if s.is_empty() {
            return err!("expected non-empty string for \
                         UnicodeDataDecomposition value");
        }
        let caps = match WITH_TAG.captures(s) {
            Some(caps) => caps,
            None => return err!("invalid decomposition value"),
        };
        let mut decomp = UnicodeDataDecomposition::default();
        let mut codepoints = s;
        if let Some(m) = caps.name("tag") {
            decomp.tag = Some(m.as_str().parse()?);
            codepoints = &caps["chars"];
        }
        for m in CHARS.find_iter(codepoints) {
            let cp = m.as_str().parse()?;
            decomp.push(cp)?;
        }
        Ok(decomp)
    }
}

impl fmt::Display for UnicodeDataDecomposition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref tag) = self.tag {
            write!(f, "<{}> ", tag)?;
        }
        let mut first = true;
        for cp in self.mapping() {
            if !first {
                write!(f, " ")?;
            }
            first = false;
            write!(f, "{}", cp)?;
        }
        Ok(())
    }
}

/// The formatting tag on a decomposition mapping.
///
/// This is taken from
/// [UAX44, Table 14](http://www.unicode.org/reports/tr44/#Character_Decomposition_Mappings).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnicodeDataDecompositionTag {
    /// <font>
    Font,
    /// <noBreak>
    NoBreak,
    /// <initial>
    Initial,
    /// <medial>
    Medial,
    /// <final>
    Final,
    /// <isolated>
    Isolated,
    /// <circle>
    Circle,
    /// <super>
    Super,
    /// <sub>
    Sub,
    /// <vertical>
    Vertical,
    /// <wide>
    Wide,
    /// <narrow>
    Narrow,
    /// <small>
    Small,
    /// <square>
    Square,
    /// <fraction>
    Fraction,
    /// <compat>
    Compat,
}

impl FromStr for UnicodeDataDecompositionTag {
    type Err = Error;

    fn from_str(s: &str) -> Result<UnicodeDataDecompositionTag, Error> {
        use self::UnicodeDataDecompositionTag::*;
        Ok(match s {
            "font" => Font,
            "noBreak" => NoBreak,
            "initial" => Initial,
            "medial" => Medial,
            "final" => Final,
            "isolated" => Isolated,
            "circle" => Circle,
            "super" => Super,
            "sub" => Sub,
            "vertical" => Vertical,
            "wide" => Wide,
            "narrow" => Narrow,
            "small" => Small,
            "square" => Square,
            "fraction" => Fraction,
            "compat" => Compat,
            _ => return err!("invalid decomposition formatting tag: {}", s),
        })
    }
}

impl fmt::Display for UnicodeDataDecompositionTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::UnicodeDataDecompositionTag::*;
        let s = match *self {
            Font => "font",
            NoBreak => "noBreak",
            Initial => "initial",
            Medial => "medial",
            Final => "final",
            Isolated => "isolated",
            Circle => "circle",
            Super => "super",
            Sub => "sub",
            Vertical => "vertical",
            Wide => "wide",
            Narrow => "narrow",
            Small => "small",
            Square => "square",
            Fraction => "fraction",
            Compat => "compat",
        };
        write!(f, "{}", s)
    }
}

/// A numeric value corresponding to characters with `Numeric_Type=Numeric`.
///
/// A numeric value can either be a signed integer or a rational number.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnicodeDataNumeric {
    /// An integer.
    Integer(i64),
    /// A rational number. The first is the numerator and the latter is the
    /// denominator.
    Rational(i64, i64),
}

impl FromStr for UnicodeDataNumeric {
    type Err = Error;

    fn from_str(s: &str) -> Result<UnicodeDataNumeric, Error> {
        if s.is_empty() {
            return err!(
                "expected non-empty string for UnicodeDataNumeric value");
        }
        if let Some(pos) = s.find('/') {
            let (snum, sden) = (&s[..pos], &s[pos+1..]);
            let num = match snum.parse() {
                Ok(num) => num,
                Err(err) => {
                    return err!(
                        "invalid integer numerator '{}': {}", snum, err);
                }
            };
            let den = match sden.parse() {
                Ok(den) => den,
                Err(err) => {
                    return err!(
                        "invalid integer denominator '{}': {}", sden, err);
                }
            };
            Ok(UnicodeDataNumeric::Rational(num, den))
        } else {
            match s.parse() {
                Ok(den) => Ok(UnicodeDataNumeric::Integer(den)),
                Err(err) => {
                    return err!(
                        "invalid integer denominator '{}': {}", s, err);
                }
            }
        }
    }
}

impl fmt::Display for UnicodeDataNumeric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UnicodeDataNumeric::Integer(n) => write!(f, "{}", n),
            UnicodeDataNumeric::Rational(n, d) => write!(f, "{}/{}", n, d),
        }
    }
}

/// An iterator adapter that expands rows in `UnicodeData.txt`.
///
/// Throughout `UnicodeData.txt`, some assigned codepoints are not explicitly
/// represented. Instead, they are represented by a pair of rows, indicating
/// a range of codepoints with the same properties. For example, the Hangul
/// syllable codepoints are represented by these two rows:
///
/// ```ignore
/// AC00;<Hangul Syllable, First>;Lo;0;L;;;;;N;;;;;
/// D7A3;<Hangul Syllable, Last>;Lo;0;L;;;;;N;;;;;
/// ```
///
/// This iterator will wrap any iterator of `UnicodeData` and, when a range of
/// Unicode codepoints is found, it will be expanded to the appropriate
/// sequence of `UnicodeData` values. Note that all such expanded records will
/// have an empty name.
pub struct UnicodeDataExpander<I: Iterator> {
    /// The underlying iterator.
    it: iter::Peekable<I>,
    /// A range of codepoints to emit when we've found a pair. Otherwise,
    /// `None`.
    range: CodepointRange,
}

struct CodepointRange {
    /// The codepoint range.
    range: Range<u32>,
    /// The start record. All subsequent records in this range are generated
    /// by cloning this and updating the codepoint/name.
    start_record: UnicodeData,
}

impl<I: Iterator<Item=UnicodeData>> UnicodeDataExpander<I> {
    /// Create a new iterator that expands pairs of `UnicodeData` range
    /// records. All other records are passed through as-is.
    pub fn new<T>(it: T) -> UnicodeDataExpander<I>
            where T: IntoIterator<IntoIter=I, Item=I::Item>
    {
        UnicodeDataExpander {
            it: it.into_iter().peekable(),
            range: CodepointRange {
                range: 0..0,
                start_record: UnicodeData::default(),
            },
        }
    }
}

impl<I: Iterator<Item=UnicodeData>>
    Iterator for UnicodeDataExpander<I>
{
    type Item = UnicodeData;

    fn next(&mut self) -> Option<UnicodeData> {
        if let Some(udata) = self.range.next() {
            return Some(udata);
        }
        let row1 = match self.it.next() {
            None => return None,
            Some(row1) => row1,
        };
        if !row1.is_range_start()
            || !self.it.peek().map_or(false, |row2| row2.is_range_end())
        {
            return Some(row1)
        }
        let row2 = self.it.next().unwrap();
        self.range = CodepointRange {
            range: row1.codepoint.value()..(row2.codepoint.value() + 1),
            start_record: row1,
        };
        self.next()
    }
}

impl Iterator for CodepointRange {
    type Item = UnicodeData;

    fn next(&mut self) -> Option<UnicodeData> {
        let cp = match self.range.next() {
            None => return None,
            Some(cp) => cp,
        };
        Some(UnicodeData {
            codepoint: Codepoint::from_u32(cp).unwrap(),
            name: "".to_string(),
            ..self.start_record.clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use common::Codepoint;

    use super::{
        UnicodeData, UnicodeDataNumeric,
        UnicodeDataDecomposition, UnicodeDataDecompositionTag,
    };

    fn codepoint(n: u32) -> Codepoint {
        Codepoint::from_u32(n).unwrap()
    }

    fn s(string: &str) -> String {
        string.to_string()
    }

    #[test]
    fn parse1() {
        let line = "249D;PARENTHESIZED LATIN SMALL LETTER B;So;0;L;<compat> 0028 0062 0029;;;;N;;;;;\n";
        let data: UnicodeData = line.parse().unwrap();
        assert_eq!(data, UnicodeData {
            codepoint: codepoint(0x249d),
            name: s("PARENTHESIZED LATIN SMALL LETTER B"),
            general_category: s("So"),
            canonical_combining_class: 0,
            bidi_class: s("L"),
            decomposition: UnicodeDataDecomposition::new(
                Some(UnicodeDataDecompositionTag::Compat),
                &[codepoint(0x28), codepoint(0x62), codepoint(0x29)],
            ).unwrap(),
            numeric_type_decimal: None,
            numeric_type_digit: None,
            numeric_type_numeric: None,
            bidi_mirrored: false,
            unicode1_name: s(""),
            iso_comment: s(""),
            simple_uppercase_mapping: None,
            simple_lowercase_mapping: None,
            simple_titlecase_mapping: None,
        });
    }

    #[test]
    fn parse2() {
        let line = "000D;<control>;Cc;0;B;;;;;N;CARRIAGE RETURN (CR);;;;\n";
        let data: UnicodeData = line.parse().unwrap();
        assert_eq!(data, UnicodeData {
            codepoint: codepoint(0x000D),
            name: s("<control>"),
            general_category: s("Cc"),
            canonical_combining_class: 0,
            bidi_class: s("B"),
            decomposition: UnicodeDataDecomposition::new(
                None, &[codepoint(0x000D)]).unwrap(),
            numeric_type_decimal: None,
            numeric_type_digit: None,
            numeric_type_numeric: None,
            bidi_mirrored: false,
            unicode1_name: s("CARRIAGE RETURN (CR)"),
            iso_comment: s(""),
            simple_uppercase_mapping: None,
            simple_lowercase_mapping: None,
            simple_titlecase_mapping: None,
        });
    }

    #[test]
    fn parse3() {
        let line = "00BC;VULGAR FRACTION ONE QUARTER;No;0;ON;<fraction> 0031 2044 0034;;;1/4;N;FRACTION ONE QUARTER;;;;\n";
        let data: UnicodeData = line.parse().unwrap();
        assert_eq!(data, UnicodeData {
            codepoint: codepoint(0x00BC),
            name: s("VULGAR FRACTION ONE QUARTER"),
            general_category: s("No"),
            canonical_combining_class: 0,
            bidi_class: s("ON"),
            decomposition: UnicodeDataDecomposition::new(
                Some(UnicodeDataDecompositionTag::Fraction),
                &[codepoint(0x31), codepoint(0x2044), codepoint(0x34)],
            ).unwrap(),
            numeric_type_decimal: None,
            numeric_type_digit: None,
            numeric_type_numeric: Some(UnicodeDataNumeric::Rational(1, 4)),
            bidi_mirrored: false,
            unicode1_name: s("FRACTION ONE QUARTER"),
            iso_comment: s(""),
            simple_uppercase_mapping: None,
            simple_lowercase_mapping: None,
            simple_titlecase_mapping: None,
        });
    }

    #[test]
    fn parse4() {
        let line = "0041;LATIN CAPITAL LETTER A;Lu;0;L;;;;;N;;;;0061;\n";
        let data: UnicodeData = line.parse().unwrap();
        assert_eq!(data, UnicodeData {
            codepoint: codepoint(0x0041),
            name: s("LATIN CAPITAL LETTER A"),
            general_category: s("Lu"),
            canonical_combining_class: 0,
            bidi_class: s("L"),
            decomposition: UnicodeDataDecomposition::new(
                None, &[codepoint(0x0041)]).unwrap(),
            numeric_type_decimal: None,
            numeric_type_digit: None,
            numeric_type_numeric: None,
            bidi_mirrored: false,
            unicode1_name: s(""),
            iso_comment: s(""),
            simple_uppercase_mapping: None,
            simple_lowercase_mapping: Some(codepoint(0x0061)),
            simple_titlecase_mapping: None,
        });
    }

    #[test]
    fn parse5() {
        let line = "0F33;TIBETAN DIGIT HALF ZERO;No;0;L;;;;-1/2;N;;;;;\n";
        let data: UnicodeData = line.parse().unwrap();
        assert_eq!(data, UnicodeData {
            codepoint: codepoint(0x0F33),
            name: s("TIBETAN DIGIT HALF ZERO"),
            general_category: s("No"),
            canonical_combining_class: 0,
            bidi_class: s("L"),
            decomposition: UnicodeDataDecomposition::new(
                None, &[codepoint(0x0F33)]).unwrap(),
            numeric_type_decimal: None,
            numeric_type_digit: None,
            numeric_type_numeric: Some(UnicodeDataNumeric::Rational(-1, 2)),
            bidi_mirrored: false,
            unicode1_name: s(""),
            iso_comment: s(""),
            simple_uppercase_mapping: None,
            simple_lowercase_mapping: None,
            simple_titlecase_mapping: None,
        });
    }

    #[test]
    fn expander() {
        use common::UcdLineParser;
        use super::UnicodeDataExpander;

        let data = "\
ABF9;MEETEI MAYEK DIGIT NINE;Nd;0;L;;9;9;9;N;;;;;
AC00;<Hangul Syllable, First>;Lo;0;L;;;;;N;;;;;
D7A3;<Hangul Syllable, Last>;Lo;0;L;;;;;N;;;;;
D7B0;HANGUL JUNGSEONG O-YEO;Lo;0;L;;;;;N;;;;;
";
        let records = UcdLineParser::new(data.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(UnicodeDataExpander::new(records).count(), 11174);
    }
}
