extern crate itertools;
use itertools::Itertools;

fn main (){
    let key_bin = include_bytes!("../key.bin");
    let key_hexstr = String::from(format!("cargo:warning=Enclave encryption key: {:02X}", key_bin.iter().format("")));
    println!("{}", key_hexstr);
}
