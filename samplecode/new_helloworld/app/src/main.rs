use edl::ecalls;
use sgx_new_edl::{In, Out};

extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::error::SgxStatus;
use sgx_types::types::*;
use sgx_urts::enclave::SgxEnclave;

static ENCLAVE_FILE: &str = "libenclave.so";

fn main() {
    let enclave = match SgxEnclave::create(ENCLAVE_FILE, true) {
        Ok(enclave) => {
            println!("[+] Init Enclave Successful {}!", enclave.eid());
            enclave
        }
        Err(err) => {
            println!("[-] Init Enclave Failed {}!", err.as_str());
            return;
        }
    };

    let input_string = String::from("This is a normal world string passed into Enclave!\n");
    let mut retval = SgxStatus::Success;

    let a1 = String::new();
    let a1 = In::new(&a1);
    let mut o1 = String::new();
    let o1 = Out::new(&mut o1);
    let o_tab = [];

    ecalls::foo::ecall(0, &o_tab, a1, o1);

    // let result = unsafe {
    //     say_something(
    //         enclave.eid(),
    //         &mut retval,
    //         input_string.as_ptr() as *const u8,
    //         input_string.len(),
    //     )
    // };
    // match result {
    //     SgxStatus::Success => println!("[+] ECall Success..."),
    //     _ => println!("[-] ECall Enclave Failed {}!", result.as_str()),
    // }
}
