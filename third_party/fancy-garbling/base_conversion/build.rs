extern crate cc;

fn main() {
    cc::Build::new().file("cbits/lookup_tables.c").compile("lookup_tables.a");
}
