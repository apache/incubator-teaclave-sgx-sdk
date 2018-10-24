extern crate cc;

fn main() {
    let mut c = cc::Build::new();
    let _ = c.file("cbits/aesni.c");
    let _ = c.flag("-Wno-unused-variable");
    let _ = c.compile("libaesni.a");
}
