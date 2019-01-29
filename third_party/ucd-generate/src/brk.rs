use std::collections::{BTreeMap, BTreeSet};

use ucd_parse::{self, GraphemeClusterBreak, SentenceBreak, WordBreak};

use args::ArgMatches;
use error::Result;

pub fn grapheme_cluster(args: ArgMatches) -> Result<()> {
    let ucd_dir = args.ucd_dir()?;
    let vals: Vec<GraphemeClusterBreak> = ucd_parse::parse(&ucd_dir)?;

    let mut byval: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    for x in &vals {
        byval
            .entry(x.value.clone())
            .or_insert(BTreeSet::new())
            .extend(x.codepoints.into_iter().map(|c| c.value()));
    }

    let mut wtr = args.writer("grapheme_cluster_break")?;
    if args.is_present("enum") {
        wtr.ranges_to_enum(args.name(), &byval)?;
    } else {
        wtr.names(byval.keys())?;
        for (val, set) in byval {
            wtr.ranges(&val, &set)?;
        }
    }
    Ok(())
}

pub fn word(args: ArgMatches) -> Result<()> {
    let ucd_dir = args.ucd_dir()?;
    let vals: Vec<WordBreak> = ucd_parse::parse(&ucd_dir)?;

    let mut byval: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    for x in &vals {
        byval
            .entry(x.value.clone())
            .or_insert(BTreeSet::new())
            .extend(x.codepoints.into_iter().map(|c| c.value()));
    }

    let mut wtr = args.writer("word_break")?;
    if args.is_present("enum") {
        wtr.ranges_to_enum(args.name(), &byval)?;
    } else {
        wtr.names(byval.keys())?;
        for (val, set) in byval {
            wtr.ranges(&val, &set)?;
        }
    }
    Ok(())
}

pub fn sentence(args: ArgMatches) -> Result<()> {
    let ucd_dir = args.ucd_dir()?;
    let vals: Vec<SentenceBreak> = ucd_parse::parse(&ucd_dir)?;

    let mut byval: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    for x in &vals {
        byval
            .entry(x.value.clone())
            .or_insert(BTreeSet::new())
            .extend(x.codepoints.into_iter().map(|c| c.value()));
    }

    let mut wtr = args.writer("sentence_break")?;
    if args.is_present("enum") {
        wtr.ranges_to_enum(args.name(), &byval)?;
    } else {
        wtr.names(byval.keys())?;
        for (val, set) in byval {
            wtr.ranges(&val, &set)?;
        }
    }
    Ok(())
}
