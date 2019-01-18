use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ucd_parse::{
    self, CoreProperty, EmojiProperty, Property, UnicodeDataExpander,
};

use args::ArgMatches;
use error::Result;
use util::{PropertyNames, PropertyValues};

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let by_name = parse_properties(&dir)?;
    let properties = PropertyNames::from_ucd_dir(&dir)?;
    let filter = args.filter(|name| properties.canonical(name))?;

    if args.is_present("list-properties") {
        for name in by_name.keys() {
            println!("{}", name);
        }
        return Ok(());
    }
    let mut wtr = args.writer("prop_list")?;
    wtr.names(by_name.keys().filter(|n| filter.contains(n)))?;
    for (name, set) in by_name {
        if filter.contains(&name) {
            wtr.ranges(&name, &set)?;
        }
    }
    Ok(())
}

pub fn command_perl_word(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let props = parse_properties(&dir)?;
    let gencats = parse_general_categories(&dir)?;

    let mut perlword = BTreeSet::new();
    perlword.extend(&props["Alphabetic"]);
    perlword.extend(&props["Join_Control"]);
    perlword.extend(&gencats["Decimal_Number"]);
    perlword.extend(&gencats["Nonspacing_Mark"]);
    perlword.extend(&gencats["Enclosing_Mark"]);
    perlword.extend(&gencats["Spacing_Mark"]);
    perlword.extend(&gencats["Connector_Punctuation"]);

    let mut wtr = args.writer("perl_word")?;
    wtr.ranges(args.name(), &perlword)?;
    Ok(())
}

fn parse_properties<P: AsRef<Path>>(
    ucd_dir: P,
) -> Result<BTreeMap<String, BTreeSet<u32>>> {
    // TODO: PropList.txt and DerivedCoreProperties.txt cover the majority
    // of boolean properties, but UAX44 S5.3 Table 9 lists a smattering of
    // others that we should include here as well. (Some will need support in
    // ucd-parse, for example, the ones found in DerivedNormalizationProps.txt
    // while others, like Bidi_Mirrored, are derived from UnicodeData.txt.
    // Even still, others like Composition_Exclusion have their own file
    // (CompositionExclusions.txt).

    let mut by_name: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();

    let prop_list: Vec<Property> = ucd_parse::parse(&ucd_dir)?;
    for x in &prop_list {
        by_name
            .entry(x.property.clone())
            .or_insert(BTreeSet::new())
            .extend(x.codepoints.into_iter().map(|c| c.value()));
    }

    let core_prop: Vec<CoreProperty> = ucd_parse::parse(&ucd_dir)?;
    for x in &core_prop {
        by_name
            .entry(x.property.clone())
            .or_insert(BTreeSet::new())
            .extend(x.codepoints.into_iter().map(|c| c.value()));
    }

    let emoji_prop: Vec<EmojiProperty> = ucd_parse::parse(&ucd_dir)?;
    for x in &emoji_prop {
        by_name
            .entry(x.property.clone())
            .or_insert(BTreeSet::new())
            .extend(x.codepoints.into_iter().map(|c| c.value()));
    }
    Ok(by_name)
}

fn parse_general_categories<P: AsRef<Path>>(
    ucd_dir: P,
) -> Result<BTreeMap<String, BTreeSet<u32>>> {
    let propvals = PropertyValues::from_ucd_dir(&ucd_dir)?;
    let unexpanded = ucd_parse::parse(&ucd_dir)?;
    // Expand all of our UnicodeData rows. This results in one big list of
    // all assigned codepoints.
    let rows: Vec<_> = UnicodeDataExpander::new(unexpanded).collect();

    // Collect each general category into an ordered set.
    let mut bycat: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    for row in rows {
        let gc = propvals
            .canonical("gc", &row.general_category)?
            .to_string();
        bycat.entry(gc)
            .or_insert(BTreeSet::new())
            .insert(row.codepoint.value());
    }
    Ok(bycat)
}
