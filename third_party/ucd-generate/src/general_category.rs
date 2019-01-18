use std::collections::{BTreeMap, BTreeSet};

use ucd_parse::{self, UnicodeDataExpander};

use args::ArgMatches;
use error::Result;
use util::{PropertyValues, print_property_values};

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let propvals = PropertyValues::from_ucd_dir(&dir)?;
    let filter = args.filter(|name| propvals.canonical("gc", name))?;
    let unexpanded = ucd_parse::parse(&dir)?;

    // If we were tasked with listing the available categories, then do that
    // and quit.
    if args.is_present("list-categories") {
        return print_property_values(&propvals, "General_Category");
    }

    // Expand all of our UnicodeData rows. This results in one big list of
    // all assigned codepoints.
    let rows: Vec<_> = UnicodeDataExpander::new(unexpanded).collect();

    // Collect each general category into an ordered set.
    let mut bycat: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    let mut assigned = BTreeSet::new();
    for row in rows {
        assigned.insert(row.codepoint.value());
        let gc = propvals
            .canonical("gc", &row.general_category)?
            .to_string();
        bycat.entry(gc)
            .or_insert(BTreeSet::new())
            .insert(row.codepoint.value());
    }
    // As a special case, collect all unassigned codepoints.
    let unassigned_name = propvals
        .canonical("gc", "unassigned")?
        .to_string();
    bycat.insert(unassigned_name.clone(), BTreeSet::new());
    for cp in 0..(0x10FFFF + 1) {
        if !assigned.contains(&cp) {
            bycat.get_mut(&unassigned_name).unwrap().insert(cp);
        }
    }
    // As another special case, collect all "related" groups of categories.
    // But don't do this when printing an enumeration, because in an
    // enumeration each codepoint should belong to exactly one category, which
    // is not true if we include related categories.
    if !args.is_present("enum") {
        for (name, set) in related(&propvals, &bycat) {
            if filter.contains(&name) {
                bycat.insert(name, set);
            }
        }
    }
    // Finally, filter out any sets according to what the user asked for.
    let bycat = bycat
        .into_iter()
        .filter(|&(ref name, _)| filter.contains(name))
        .collect();

    let mut wtr = args.writer("general_category")?;
    if args.is_present("enum") {
        wtr.ranges_to_enum(args.name(), &bycat)?;
    } else {
        wtr.names(bycat.keys().filter(|n| filter.contains(n)))?;
        for (name, set) in bycat {
            wtr.ranges(&name, &set)?;
        }
    }

    Ok(())
}

/// Related returns a set of sets of codepoints corresponding to the "related"
/// groups of categories defined by Table 12 in UAX#44 S5.7.1.
///
/// The given `cats` should correspond to the normal set of general categories,
/// keyed by canonical name.
fn related(
    propvals: &PropertyValues,
    cats: &BTreeMap<String, BTreeSet<u32>>,
) -> BTreeMap<String, BTreeSet<u32>> {
    let mut sets = BTreeMap::new();
    for (name, components) in related_categories(propvals) {
        let mut set = sets.entry(name).or_insert(BTreeSet::new());
        for component in components {
            set.extend(cats[&component].iter().cloned());
        }
    }
    sets
}

/// Return all groups of "related" general categories.
fn related_categories(
    propvals: &PropertyValues,
) -> Vec<(String, Vec<String>)> {
    // canonicalize a gencat property value
    let c = |name: &str| -> String {
        propvals.canonical("gc", name).unwrap().to_string()
    };
    vec![
        (c("Cased_Letter"), vec![c("lu"), c("ll"), c("lt")]),
        (c("Letter"), vec![c("lu"), c("ll"), c("lt"), c("lm"), c("lo")]),
        (c("Mark"), vec![c("mn"), c("mc"), c("me")]),
        (c("Number"), vec![c("nd"), c("nl"), c("no")]),
        (c("Punctuation"), vec![
            c("pc"), c("pd"), c("ps"), c("pe"), c("pi"), c("pf"), c("po"),
        ]),
        (c("Symbol"), vec![c("sm"), c("sc"), c("sk"), c("so")]),
        (c("Separator"), vec![c("zs"), c("zl"), c("zp")]),
        (c("Other"), vec![c("cc"), c("cf"), c("cs"), c("co"), c("cn")]),
    ]
}
