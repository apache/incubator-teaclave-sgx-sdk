use http_req::request;

fn main() {
    let res = request::head("https://www.rust-lang.org/learn").unwrap();

    println!("Status: {} {}", res.status_code(), res.reason());
    println!("{:?}", res.headers());
}
