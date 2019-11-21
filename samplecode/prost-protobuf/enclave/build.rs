use std::path::PathBuf;

fn main () {
    let src = PathBuf::from("..");
    let includes = &[src.clone()];
    let mut config = prost_build::Config::new();

    config.compile_protos(&[src.join("person.proto")], includes).unwrap();
}
