use http_req::request;

fn main() {
    let mut writer = Vec::new(); //container for body of a response
    let res = request::get("https://www.rust-lang.org/learn", &mut writer).unwrap();

    println!("Status: {} {}", res.status_code(), res.reason());
    println!("Headers {}", res.headers());
    //println!("{}", String::from_utf8_lossy(&writer));
}
