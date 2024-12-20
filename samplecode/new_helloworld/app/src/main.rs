use common::ecalls;
use sgx_new_edl::args::{In, Out};

fn main() {
    let eid = 0;
    let o_tab = [];
    let a1 = String::new();
    let mut o1 = String::new();
    ecalls::hello_world::ecall(eid, &o_tab, In::new(&a1), Out::new(&mut o1));
    todo!();
    println!("Hello, world!");
}
