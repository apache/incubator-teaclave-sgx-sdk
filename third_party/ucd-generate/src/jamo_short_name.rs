use std::collections::BTreeMap;

use ucd_parse::{self, JamoShortName};

use args::ArgMatches;
use error::Result;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let jamo_map = ucd_parse::parse_by_codepoint::<_, JamoShortName>(dir)?;

    let mut wtr = args.writer("jamo_short_name")?;
    let mut map = BTreeMap::new();
    for (cp, jamo) in jamo_map {
        map.insert(cp.value(), jamo.name);
    }
    wtr.codepoint_to_string(args.name(), &map)?;
    Ok(())
}
