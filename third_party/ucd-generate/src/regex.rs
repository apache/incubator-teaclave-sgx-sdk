use args::ArgMatches;
use error::Result;

use regex_automata::{dense, RegexBuilder};

pub fn command_dfa(args: ArgMatches) -> Result<()> {
    let mut wtr = args.dfa_writer(args.name())?;
    let pattern = match args.value_of("pattern") {
        None => return err!("missing regex pattern"),
        Some(pattern) => pattern,
    };

    let dfa = dfa_builder(&args).build(&pattern)?;
    if args.is_present("sparse") {
        match state_size(&args) {
            1 => {
                let dfa = dfa.to_u8()?.to_sparse()?;
                wtr.sparse_dfa(args.name(), &dfa)?;
            }
            2 => {
                let dfa = dfa.to_u16()?.to_sparse()?;
                wtr.sparse_dfa(args.name(), &dfa)?;
            }
            4 => {
                let dfa = dfa.to_u32()?.to_sparse()?;
                wtr.sparse_dfa(args.name(), &dfa)?;
            }
            8 => {
                let dfa = dfa.to_u64()?.to_sparse()?;
                wtr.sparse_dfa(args.name(), &dfa)?;
            }
            _ => unreachable!(),
        }
    } else {
        match state_size(&args) {
            1 => {
                let dfa = dfa.to_u8()?;
                wtr.dense_dfa(args.name(), &dfa)?;
            }
            2 => {
                let dfa = dfa.to_u16()?;
                wtr.dense_dfa(args.name(), &dfa)?;
            }
            4 => {
                let dfa = dfa.to_u32()?;
                wtr.dense_dfa(args.name(), &dfa)?;
            }
            8 => {
                let dfa = dfa.to_u64()?;
                wtr.dense_dfa(args.name(), &dfa)?;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub fn command_regex(args: ArgMatches) -> Result<()> {
    let mut wtr = args.dfa_writer(args.name())?;
    let pattern = match args.value_of("pattern") {
        None => return err!("missing regex pattern"),
        Some(pattern) => pattern,
    };

    let builder = regex_builder(&args);
    if args.is_present("sparse") {
        match state_size(&args) {
            1 => {
                let re = builder.build_with_size_sparse::<u8>(&pattern)?;
                wtr.sparse_regex(args.name(), &re)?;
            }
            2 => {
                let re = builder.build_with_size_sparse::<u16>(&pattern)?;
                wtr.sparse_regex(args.name(), &re)?;
            }
            4 => {
                let re = builder.build_with_size_sparse::<u32>(&pattern)?;
                wtr.sparse_regex(args.name(), &re)?;
            }
            8 => {
                let re = builder.build_with_size_sparse::<u64>(&pattern)?;
                wtr.sparse_regex(args.name(), &re)?;
            }
            _ => unreachable!(),
        }
    } else {
        match state_size(&args) {
            1 => {
                let re = builder.build_with_size::<u8>(&pattern)?;
                wtr.dense_regex(args.name(), &re)?;
            }
            2 => {
                let re = builder.build_with_size::<u16>(&pattern)?;
                wtr.dense_regex(args.name(), &re)?;
            }
            4 => {
                let re = builder.build_with_size::<u32>(&pattern)?;
                wtr.dense_regex(args.name(), &re)?;
            }
            8 => {
                let re = builder.build_with_size::<u64>(&pattern)?;
                wtr.dense_regex(args.name(), &re)?;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn regex_builder(args: &ArgMatches) -> RegexBuilder {
    let mut builder = RegexBuilder::new();
    builder
        .minimize(args.is_present("minimize"))
        .anchored(args.is_present("anchored"))
        .byte_classes(args.is_present("classes"))
        .premultiply(args.is_present("premultiply"))
        .allow_invalid_utf8(args.is_present("no-utf8"));
    builder
}

fn dfa_builder(args: &ArgMatches) -> dense::Builder {
    let mut builder = dense::Builder::new();
    builder
        .minimize(args.is_present("minimize"))
        .anchored(args.is_present("anchored"))
        .byte_classes(args.is_present("classes"))
        .premultiply(args.is_present("premultiply"))
        .allow_invalid_utf8(args.is_present("no-utf8"))
        .longest_match(args.is_present("longest"))
        .reverse(args.is_present("reverse"));
    builder
}

fn state_size(args: &ArgMatches) -> u8 {
    // These unwraps are OK because clap should verify that there exists a
    // value and it must be 1, 2, 4 or 8.
    args.value_of("state-size").unwrap().parse().unwrap()
}
