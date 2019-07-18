//!Simple HTTP client with built-in HTTPS support.
//!Currently it's in heavy development and may frequently change.
//!
//!## Example
//!Basic GET request
//!```
//!use http_req::request;
//!
//!fn main() {
//!    let mut writer = Vec::new(); //container for body of a response
//!    let res = request::get("https://doc.rust-lang.org/", &mut writer).unwrap();
//!
//!    println!("Status: {} {}", res.status_code(), res.reason());
//!}
//!```

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

pub mod error;
pub mod request;
pub mod response;
pub mod tls;
pub mod uri;
