use std::collections::{BTreeMap, BTreeSet};

use ucd_parse::{self, Age};

use args::ArgMatches;
use error::Result;
use util::PropertyValues;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let propvals = PropertyValues::from_ucd_dir(&dir)?;
    let ages: Vec<Age> = ucd_parse::parse(&dir)?;

    let mut by_age: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    for x in &ages {
        let agename = propvals.canonical("Age", &x.age)?;
        by_age
            .entry(agename)
            .or_insert(BTreeSet::new())
            .extend(x.codepoints.into_iter().map(|c| c.value()));
    }

    let mut wtr = args.writer("age")?;
    wtr.names(by_age.keys())?;
    for (name, set) in by_age {
        wtr.ranges(&name, &set)?;
    }
    Ok(())
}
