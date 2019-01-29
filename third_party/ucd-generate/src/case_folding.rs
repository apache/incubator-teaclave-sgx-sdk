use std::collections::{BTreeMap, BTreeSet};

use ucd_parse::{self, CaseFold, CaseStatus, Codepoint};

use args::ArgMatches;
use error::Result;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let case_folding: BTreeMap<Codepoint, Vec<CaseFold>> =
        ucd_parse::parse_many_by_codepoint(dir)?;

    let compute_all_pairs =
        args.is_present("all-pairs") || args.is_present("circular");
    let mut wtr = args.writer("case_folding_simple")?;
    let mut table = BTreeMap::new();
    let mut table_all = BTreeMap::new();
    for (&cp, case_folds) in &case_folding {
        let mapping_cp = match choose_fold(case_folds, false)? {
            None => continue,
            Some(case_fold) => &case_fold.mapping,
        };
        assert_eq!(mapping_cp.len(), 1);

        let (a, b) = (cp.value(), mapping_cp[0].value());
        table.insert(a, b);
        if compute_all_pairs {
            table_all.entry(a).or_insert(BTreeSet::new()).insert(b);
            table_all.entry(b).or_insert(BTreeSet::new()).insert(a);
        }
    }
    if compute_all_pairs {
        let mut exhaustive = BTreeMap::new();
        for (&k, vs) in &table_all {
            exhaustive.insert(k, BTreeSet::new());
            for &v in vs {
                exhaustive.get_mut(&k).unwrap().insert(v);
                if let Some(vs2) = table_all.get(&v) {
                    for &v2 in vs2 {
                        exhaustive.get_mut(&k).unwrap().insert(v2);
                    }
                }
            }
            exhaustive.get_mut(&k).unwrap().remove(&k);
        }
        table_all = exhaustive;
    }

    if args.is_present("circular") {
        let mut equiv = BTreeMap::new();
        let mut seen = BTreeSet::new();
        for (&k, vs) in &table_all {
            if vs.is_empty() || seen.contains(&k) {
                continue;
            }
            seen.insert(k);
            for &v in vs {
                seen.insert(v);
            }

            let mut cur = *vs.iter().last().unwrap();
            for &v in Some(&k).into_iter().chain(vs.iter()) {
                assert!(!equiv.contains_key(&cur));
                equiv.insert(cur, v);
                cur = v;
            }
        }
        wtr.codepoint_to_codepoint(args.name(), &equiv)?;
    } else if args.is_present("all-pairs") {
        wtr.multi_codepoint_to_codepoint(args.name(), &table_all)?;
    } else {
        wtr.codepoint_to_codepoint(args.name(), &table)?;
    }
    Ok(())
}

/// Given a sequence of case fold mappings, choose exactly one mapping based
/// on the mapping's status. If `full` is true, then full case mappings are
/// selected, otherwise simple case mappings are selected. If there are
/// multiple valid choices, then an error is returned.
fn choose_fold(
    case_folds: &[CaseFold],
    full: bool,
) -> Result<Option<&CaseFold>> {
    let mut choice = None;
    for case_fold in case_folds {
        if (full && case_fold.status == CaseStatus::Full)
            || (!full && case_fold.status == CaseStatus::Simple)
            || case_fold.status == CaseStatus::Common
        {
            if choice.is_some() {
                return err!("found multiple matches from: {:?}", case_folds);
            }
            choice = Some(case_fold);
        }
    }
    Ok(choice)
}
