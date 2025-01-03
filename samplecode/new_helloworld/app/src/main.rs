use edl::ecalls;
use sgx_new_edl::{In, Out};

fn main() {
    let eid = 0;
    let a1 = String::new();
    let a1 = In::new(&a1);
    let mut o1 = String::new();
    let o1 = Out::new(&mut o1);
    let o_tab = [];
    ecalls::foo::ecall(eid, &o_tab, a1, o1);
    println!("Hello, world!");
}
