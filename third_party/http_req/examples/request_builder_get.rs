use http_req::{request::RequestBuilder, tls::TlsClient, uri::Uri};

fn main() {
    let addr: Uri = "https://example.com/".parse().unwrap();
    let mut tlsclient = TlsClient::new(&addr);

    //Container for response's body
    let mut writer = Vec::new();

    //Add header `Connection: Close`
    let mut request = RequestBuilder::new(&addr);
    let request = request.header("Connection", "Close");
    request.send(&mut tlsclient, &mut writer).unwrap();

    tlsclient.pool_response();

    let response = request.receive(&mut tlsclient, &mut writer).unwrap();

    println!("Status: {} {}", response.status_code(), response.reason());
    println!("{}", String::from_utf8_lossy(&writer));
}
