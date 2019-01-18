#![feature(test)]

extern crate byteorder;
extern crate fst;
#[macro_use]
extern crate lazy_static;
extern crate test;
extern crate ucd_trie;
extern crate ucd_util;

use std::cmp::Ordering;

use byteorder::{ByteOrder, BigEndian as BE};
use test::Bencher;

mod tables;

fn u32_key(cp: u32) -> [u8; 4] {
    let mut key = [0; 4];
    BE::write_u32(&mut key, cp);
    key
}

#[bench]
fn general_category_slice(b: &mut Bencher) {
    let slice = tables::slice::general_category::GENERAL_CATEGORY;
    let mut i = 0;
    b.iter(|| {
        let (query, _, value) = slice[i];
        i = (i + 1) % slice.len();

        let pos = slice.binary_search_by(|&(s, e, _)| {
            if s > query {
                Ordering::Greater
            } else if e < query {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        let found = slice[pos.unwrap()];
        assert_eq!(found.2, value);
    });
}

#[bench]
fn general_category_fst(b: &mut Bencher) {
    let slice = tables::slice::general_category::GENERAL_CATEGORY;
    let fst = &tables::fst::general_category::GENERAL_CATEGORY;

    let mut i = 0;
    b.iter(|| {
        let (query, _, value) = slice[i];
        i = (i + 1) % slice.len();

        let found = fst.get(u32_key(query)).unwrap() as u8;
        assert_eq!(found, value);
    });
}

#[bench]
fn lowercase_letter_slice(b: &mut Bencher) {
    let slice = tables::slice::general_categories::LOWERCASE_LETTER;
    let mut i = 0;
    b.iter(|| {
        let (query, _) = slice[i];
        i = (i + 1) % slice.len();

        let pos = slice.binary_search_by(|&(s, e)| {
            if s > query {
                Ordering::Greater
            } else if e < query {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        assert!(pos.is_ok());
    });
}

#[bench]
fn lowercase_letter_trie(b: &mut Bencher) {
    let slice = tables::slice::general_categories::LOWERCASE_LETTER;
    let trie = tables::trie::general_categories::LOWERCASE_LETTER;
    let mut i = 0;
    b.iter(|| {
        let (query, _) = slice[i];
        i = (i + 1) % slice.len();
        assert!(trie.contains_u32(query));
    });
}

#[bench]
fn names_slice(b: &mut Bencher) {
    let slice = tables::slice::names::NAMES;
    let mut i = 0;
    b.iter(|| {
        let (name, cp) = slice[i];
        i = (i + 1) % slice.len();

        let found = slice[slice.binary_search_by_key(&name, |x| x.0).unwrap()];
        assert_eq!(found.1, cp);
    });
}

#[bench]
fn names_fst(b: &mut Bencher) {
    let slice = tables::slice::names::NAMES;
    let fst = &tables::fst::names::NAMES;

    let mut i = 0;
    b.iter(|| {
        let (name, cp) = slice[i];
        i = (i + 1) % slice.len();

        let found = fst.get(name).unwrap() as u32;
        assert_eq!(found, cp);
    });
}

#[bench]
fn jamo_short_name_fst(b: &mut Bencher) {
    let slice = tables::slice::jamo_short_name::JAMO_SHORT_NAME;
    let fst = &tables::fst::jamo_short_name::JAMO_SHORT_NAME;
    let mut i = 0;
    let mut value = String::new();
    b.iter(|| {
        let (cp, name) = slice[i];
        i = (i + 1) % slice.len();

        let mut found = fst.get(u32_key(cp)).unwrap();
        value.clear();
        while found != 0 {
            value.push((found & 0xFF) as u8 as char);
            found = found >> 8;
        }
        assert_eq!(value, name);
    });
}

#[bench]
fn jamo_short_name_slice(b: &mut Bencher) {
    let slice = tables::slice::jamo_short_name::JAMO_SHORT_NAME;
    let mut i = 0;
    b.iter(|| {
        let (cp, name) = slice[i];
        i = (i + 1) % slice.len();

        let found = slice[slice.binary_search_by_key(&cp, |x| x.0).unwrap()];
        assert_eq!(found.1, name);
    });
}

#[bench]
fn jamo_short_name_slice_linear(b: &mut Bencher) {
    let slice = tables::slice::jamo_short_name::JAMO_SHORT_NAME;
    let mut i = 0;
    b.iter(|| {
        let (cp, name) = slice[i];
        i = (i + 1) % slice.len();

        let found = slice.iter().find(|p| p.0 == cp).unwrap();
        assert_eq!(found.1, name);
    });
}
