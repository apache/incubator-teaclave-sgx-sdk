#![feature(test)]
extern crate http_req;
extern crate test;

use http_req::{request::RequestBuilder, response::Response, uri::Uri};
use std::{fs::File, io::Read};
use test::Bencher;

#[bench]
fn parse_response(b: &mut Bencher) {
    let mut content = Vec::new();
    let mut response = File::open("benches/res.txt").unwrap();
    response.read_to_end(&mut content).unwrap();

    b.iter(|| {
        let mut body = Vec::new();
        Response::try_from(&content, &mut body)
    });
}

const URI: &str = "https://doc.rust-lang.org/stable/std/string/struct.String.html";

#[bench]
fn request_builder_send(b: &mut Bencher) {
    let mut reader = File::open("benches/res.txt").unwrap();

    b.iter(|| {
        let uri = URI.parse::<Uri>().unwrap();
        let mut writer = Vec::new();

        RequestBuilder::new(&uri).send(&mut reader, &mut writer)
    });
}

#[bench]
fn parse_uri(b: &mut Bencher) {
    b.iter(|| URI.parse::<Uri>());
}
