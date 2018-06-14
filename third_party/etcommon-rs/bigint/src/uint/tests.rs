// Copyright 2017 ETC Dev Team
// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Code derived from original work by Andrew Poelstra <apoelstra@wpsoftware.net>

// Rust Bitcoin Library
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

use uint::{U128, U256, U512};
use std::str::FromStr;
use super::FromDecStrErr;

#[test]
pub fn u256_mul() {
    assert_eq!(
        U256::from_str("2").unwrap() * U256::from_str("3").unwrap(),
        U256::from_str("6").unwrap());
    assert_eq!(
        U256::from_str("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap() *
        U256::from_str("2").unwrap(),
        U256::from_str("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE").unwrap());
}

#[test]
pub fn u256_div() {
    assert_eq!(
        U256::from_str("3").unwrap() / U256::from_str("2").unwrap(),
        U256::from_str("1").unwrap());
    assert_eq!(
        U256::from_str("0x3B9ACA01").unwrap() / U256::from_str("2").unwrap(),
        U256::from_str("0x1DCD6500").unwrap());
    assert_eq!(
        U256::from_str("0xFFFFFFFD").unwrap() / U256::from_str("2").unwrap(),
        U256::from_str("0x7FFFFFFE").unwrap());
    assert_eq!(
        U256::from_str("0xFFFFFFFFFFFFFFFD").unwrap() / U256::from_str("2").unwrap(),
        U256::from_str("0x7FFFFFFFFFFFFFFE").unwrap());
    assert_eq!(
        U256::from_str("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFD").unwrap() / U256::from_str("2").unwrap(),
        U256::from_str("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE").unwrap());
}

#[test]
pub fn uint256_from() {
    let e = U256([10, 0, 0, 0]);

    // test unsigned initialization
    let ua = U256::from(10u8);
    let ub = U256::from(10u16);
    let uc =  U256::from(10u32);
    let ud = U256::from(10u64);
    assert_eq!(e, ua);
    assert_eq!(e, ub);
    assert_eq!(e, uc);
    assert_eq!(e, ud);

    // test initialization from bytes
    let va = U256::from(&[10u8][..]);
    assert_eq!(e, va);

    // more tests for initialization from bytes
    assert_eq!(U256([0x1010, 0, 0, 0]), U256::from(&[0x10u8, 0x10][..]));
    assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from(&[0x12u8, 0xf0][..]));
    assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from(&[0, 0x12u8, 0xf0][..]));
    assert_eq!(U256([0x12f0, 0 , 0, 0]), U256::from(&[0, 0, 0, 0, 0, 0, 0, 0x12u8, 0xf0][..]));
    assert_eq!(U256([0x12f0, 1 , 0, 0]), U256::from(&[1, 0, 0, 0, 0, 0, 0, 0x12u8, 0xf0][..]));
    assert_eq!(
    U256([0x12f0, 1 , 0x0910203040506077, 0x8090a0b0c0d0e0f0]),
    U256::from(&
        [
            0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0,
            0x09, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x77,
            0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0x12u8, 0xf0
        ][..]
    )
    );
    assert_eq!(
    U256([0x00192437100019fa, 0x243710, 0, 0]),
    U256::from(&[0x24u8, 0x37, 0x10,0, 0x19, 0x24, 0x37, 0x10, 0, 0x19, 0xfa][..])
    );

    // test initializtion from string
    let sa = U256::from_str("0a").unwrap();
    assert_eq!(e, sa);
    assert_eq!(U256([0x1010, 0, 0, 0]), U256::from_str("1010").unwrap());
    assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from_str("12f0").unwrap());
    assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from_str("12f0").unwrap());
    assert_eq!(U256([0x12f0, 0 , 0, 0]), U256::from_str("0000000012f0").unwrap());
    assert_eq!(U256([0x12f0, 1 , 0, 0]), U256::from_str("0100000000000012f0").unwrap());
    assert_eq!(
    U256([0x12f0, 1 , 0x0910203040506077, 0x8090a0b0c0d0e0f0]),
    U256::from_str("8090a0b0c0d0e0f00910203040506077000000000000000100000000000012f0").unwrap()
    );
}

#[test]
pub fn uint256_to() {
    let hex = "8090a0b0c0d0e0f00910203040506077583a2cf8264910e1436bda32571012f0";
    let uint = U256::from_str(hex).unwrap();
    let mut bytes = [0u8; 32];
    uint.to_big_endian(&mut bytes);
    let uint2 = U256::from(&bytes[..]);
    assert_eq!(uint, uint2);
}

#[test]
pub fn uint256_bits_test() {
    assert_eq!(U256::from(0u64).bits(), 0);
    assert_eq!(U256::from(255u64).bits(), 8);
    assert_eq!(U256::from(256u64).bits(), 9);
    assert_eq!(U256::from(300u64).bits(), 9);
    assert_eq!(U256::from(60000u64).bits(), 16);
    assert_eq!(U256::from(70000u64).bits(), 17);

    //// Try to read the following lines out loud quickly
    let mut shl = U256::from(70000u64);
    shl = shl << 100;
    assert_eq!(shl.bits(), 117);
    shl = shl << 100;
    assert_eq!(shl.bits(), 217);
    shl = shl << 100;
    assert_eq!(shl.bits(), 0);

    //// Bit set check
    //// 01010
    assert!(!U256::from(10u8).bit(0));
    assert!(U256::from(10u8).bit(1));
    assert!(!U256::from(10u8).bit(2));
    assert!(U256::from(10u8).bit(3));
    assert!(!U256::from(10u8).bit(4));

    //// byte check
    assert_eq!(U256::from(10u8).byte(0), 10);
    assert_eq!(U256::from(0xffu64).byte(0), 0xff);
    assert_eq!(U256::from(0xffu64).byte(1), 0);
    assert_eq!(U256::from(0x01ffu64).byte(0), 0xff);
    assert_eq!(U256::from(0x01ffu64).byte(1), 0x1);
    assert_eq!(U256([0u64, 0xfc, 0, 0]).byte(8), 0xfc);
    assert_eq!(U256([0u64, 0, 0, u64::max_value()]).byte(31), 0xff);
    assert_eq!(U256([0u64, 0, 0, (u64::max_value() >> 8) + 1]).byte(31), 0x01);
}

#[test]
#[cfg_attr(feature="dev", allow(eq_op))]
pub fn uint256_comp_test() {
    let small = U256([10u64, 0, 0, 0]);
    let big = U256([0x8C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]);
    let bigger = U256([0x9C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]);
    let biggest = U256([0x5C8C3EE70C644118u64, 0x0209E7378231E632, 0, 1]);

    assert!(small < big);
    assert!(big < bigger);
    assert!(bigger < biggest);
    assert!(bigger <= biggest);
    assert!(biggest <= biggest);
    assert!(bigger >= big);
    assert!(bigger >= small);
    assert!(small <= small);
}

#[test]
pub fn uint256_arithmetic_test() {
    let init = U256::from(0xDEADBEEFDEADBEEFu64);
    let copy = init;

    let add = init + copy;
    assert_eq!(add, U256([0xBD5B7DDFBD5B7DDEu64, 1, 0, 0]));
    // Bitshifts
    let shl = add << 88;
    assert_eq!(shl, U256([0u64, 0xDFBD5B7DDE000000, 0x1BD5B7D, 0]));
    let shr = shl >> 40;
    assert_eq!(shr, U256([0x7DDE000000000000u64, 0x0001BD5B7DDFBD5B, 0, 0]));
    // Increment
    let incr = shr + U256::from(1u64);
    assert_eq!(incr, U256([0x7DDE000000000001u64, 0x0001BD5B7DDFBD5B, 0, 0]));
    // Subtraction
    let sub = overflowing!(incr.overflowing_sub(init));
    assert_eq!(sub, U256([0x9F30411021524112u64, 0x0001BD5B7DDFBD5A, 0, 0]));
    // Multiplication
    let mult = sub.mul_u32(300);
    assert_eq!(mult, U256([0x8C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]));
    // Division
    assert_eq!(U256::from(105u8) / U256::from(5u8), U256::from(21u8));
    let div = mult / U256::from(300u16);
    assert_eq!(div, U256([0x9F30411021524112u64, 0x0001BD5B7DDFBD5A, 0, 0]));

    let a = U256::from_str("ff000000000000000000000000000000000000000000000000000000000000d1").unwrap();
    let b = U256::from_str("00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff2e").unwrap();
    println!("{:x}", a);
    println!("{:x}", b);
    assert_eq!(!a, b);
    assert_eq!(a, !b);
}

#[test]
pub fn uint256_simple_mul() {
    let a = U256::from_str("10000000000000000").unwrap();
    let b = U256::from_str("10000000000000000").unwrap();

    let c = U256::from_str("100000000000000000000000000000000").unwrap();
    println!("Multiplying");
    let result = a.overflowing_mul(b);
    println!("Got result");
    assert_eq!(result, (c, false))
}

#[test]
pub fn uint256_extreme_bitshift_test() {
    //// Shifting a u64 by 64 bits gives an undefined value, so make sure that
    //// we're doing the Right Thing here
    let init = U256::from(0xDEADBEEFDEADBEEFu64);

    assert_eq!(init << 64, U256([0, 0xDEADBEEFDEADBEEF, 0, 0]));
    let add = (init << 64) + init;
    assert_eq!(add, U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0]));
    assert_eq!(add >> 0, U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0]));
    assert_eq!(add << 0, U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0]));
    assert_eq!(add >> 64, U256([0xDEADBEEFDEADBEEF, 0, 0, 0]));
    assert_eq!(add << 64, U256([0, 0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0]));
}

#[test]
pub fn uint256_exp10() {
    assert_eq!(U256::exp10(0), U256::from(1u64));
    println!("\none: {:?}", U256::from(1u64));
    println!("ten: {:?}", U256::from(10u64));
    assert_eq!(U256::from(2u64) * U256::from(10u64), U256::from(20u64));
    assert_eq!(U256::exp10(1), U256::from(10u64));
    assert_eq!(U256::exp10(2), U256::from(100u64));
    assert_eq!(U256::exp10(5), U256::from(100000u64));
}

#[test]
pub fn uint256_mul32() {
    assert_eq!(U256::from(0u64).mul_u32(2), U256::from(0u64));
    assert_eq!(U256::from(1u64).mul_u32(2), U256::from(2u64));
    assert_eq!(U256::from(10u64).mul_u32(2), U256::from(20u64));
    assert_eq!(U256::from(10u64).mul_u32(5), U256::from(50u64));
    assert_eq!(U256::from(1000u64).mul_u32(50), U256::from(50000u64));
}

#[test]
fn uint256_pow() {
    assert_eq!(U256::from(10).pow(U256::from(0)), U256::from(1));
    assert_eq!(U256::from(10).pow(U256::from(1)), U256::from(10));
    assert_eq!(U256::from(10).pow(U256::from(2)), U256::from(100));
    assert_eq!(U256::from(10).pow(U256::from(3)), U256::from(1000));
    assert_eq!(U256::from(10).pow(U256::from(20)), U256::exp10(20));
}

#[test]
#[should_panic]
fn uint256_pow_overflow_panic() {
    U256::from(2).pow(U256::from(0x100));
}

#[test]
fn uint256_overflowing_pow() {
    // assert_eq!(
    // 	U256::from(2).overflowing_pow(U256::from(0xff)),
    // 	(U256::from_str("8000000000000000000000000000000000000000000000000000000000000000").unwrap(), false)
    // );
    assert_eq!(
    U256::from(2).overflowing_pow(U256::from(0x100)),
    (U256::zero(), true)
    );
}

#[test]
pub fn uint256_mul1() {
    assert_eq!(U256::from(1u64) * U256::from(10u64), U256::from(10u64));
}

#[test]
pub fn uint256_mul2() {
    let a = U512::from_str("10000000000000000fffffffffffffffe").unwrap();
    let b = U512::from_str("ffffffffffffffffffffffffffffffff").unwrap();

    assert_eq!(a * b, U512::from_str("10000000000000000fffffffffffffffcffffffffffffffff0000000000000002").unwrap());
}

#[test]
pub fn uint256_overflowing_mul() {
    assert_eq!(
    U256::from_str("100000000000000000000000000000000").unwrap().overflowing_mul(
        U256::from_str("100000000000000000000000000000000").unwrap()
    ),
    (U256::zero(), true)
    );
}

#[test]
pub fn uint128_add() {
    assert_eq!(
    U128::from_str("fffffffffffffffff").unwrap() + U128::from_str("fffffffffffffffff").unwrap(),
    U128::from_str("1ffffffffffffffffe").unwrap()
    );
}

#[test]
pub fn uint128_add_overflow() {
    assert_eq!(
    U128::from_str("ffffffffffffffffffffffffffffffff").unwrap()
        .overflowing_add(
            U128::from_str("ffffffffffffffffffffffffffffffff").unwrap()
        ),
    (U128::from_str("fffffffffffffffffffffffffffffffe").unwrap(), true)
    );
}

#[test]
#[should_panic]
// overflows panic only in debug builds. Running this test with `--release` flag, always fails
#[ignore]
pub fn uint128_add_overflow_panic() {
    U128::from_str("ffffffffffffffffffffffffffffffff").unwrap()
        +
        U128::from_str("ffffffffffffffffffffffffffffffff").unwrap();
}

#[test]
pub fn uint128_mul() {
    assert_eq!(
    U128::from_str("fffffffff").unwrap() * U128::from_str("fffffffff").unwrap(),
    U128::from_str("ffffffffe000000001").unwrap());
}

#[test]
pub fn uint512_mul() {
    assert_eq!(
    U512::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
        *
        U512::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap(),
    U512::from_str("3fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000000000001").unwrap()
    );
}

#[test]
pub fn uint256_mul_overflow() {
    assert_eq!(
    U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
        .overflowing_mul(
            U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
        ),
    (U256::from_str("1").unwrap(), true)
    );
}

#[test]
#[should_panic]
pub fn uint256_mul_overflow_panic() {
    U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
        *
        U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
}

#[test]
pub fn uint256_sub_overflow() {
    assert_eq!(
    U256::from_str("0").unwrap()
        .overflowing_sub(
            U256::from_str("1").unwrap()
        ),
    (U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap(), true)
    );
}

#[test]
#[should_panic]
pub fn uint256_sub_overflow_panic() {
    U256::from_str("0").unwrap()
        -
        U256::from_str("1").unwrap();
}

#[test]
pub fn uint256_shl() {
    assert_eq!(
    U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
        << 4,
    U256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0").unwrap()
    );
}

#[test]
pub fn uint256_shl_words() {
    assert_eq!(
    U256::from_str("0000000000000001ffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
        << 64,
    U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000").unwrap()
    );
    assert_eq!(
    U256::from_str("0000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
        << 64,
    U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000").unwrap()
    );
}

#[test]
pub fn uint256_mul() {
    assert_eq!(
    U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
        *
        U256::from_str("2").unwrap(),
    U256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe").unwrap()
    );
}

#[test]
fn uint256_div() {
    assert_eq!(U256::from(10u64) /  U256::from(1u64), U256::from(10u64));
    assert_eq!(U256::from(10u64) /  U256::from(2u64), U256::from(5u64));
    assert_eq!(U256::from(10u64) /  U256::from(3u64), U256::from(3u64));
}

#[test]
fn uint256_rem() {
    assert_eq!(U256::from(10u64) % U256::from(1u64), U256::from(0u64));
    assert_eq!(U256::from(10u64) % U256::from(3u64), U256::from(1u64));
}

#[test]
fn uint256_from_dec_str() {
    assert_eq!(U256::from_dec_str("10").unwrap(), U256::from(10u64));
    assert_eq!(U256::from_dec_str("1024").unwrap(), U256::from(1024u64));
    assert_eq!(U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639936"), Err(FromDecStrErr::InvalidLength));
    assert_eq!(U256::from_dec_str("0x11"), Err(FromDecStrErr::InvalidCharacter));
}

#[test]
fn display_uint() {
    let s = "12345678987654321023456789";
    assert_eq!(format!("{}", U256::from_dec_str(s).unwrap()), s);
}

#[test]
fn display_uint_zero() {
    assert_eq!(format!("{}", U256::from(0)), "0");
}

#[test]
fn u512_multi_adds() {
    let (result, _) = U512([0, 0, 0, 0, 0, 0, 0, 0]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, 0]));
    assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 0, 0]));

    let (result, _) = U512([1, 0, 0, 0, 0, 0, 0, 1]).overflowing_add(U512([1, 0, 0, 0, 0, 0, 0, 1]));
    assert_eq!(result, U512([2, 0, 0, 0, 0, 0, 0, 2]));

    let (result, _) = U512([0, 0, 0, 0, 0, 0, 0, 1]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, 1]));
    assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 0, 2]));

    let (result, _) = U512([0, 0, 0, 0, 0, 0, 2, 1]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 3, 1]));
    assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 5, 2]));

    let (result, _) = U512([1, 2, 3, 4, 5, 6, 7, 8]).overflowing_add(U512([9, 10, 11, 12, 13, 14, 15, 16]));
    assert_eq!(result, U512([10, 12, 14, 16, 18, 20, 22, 24]));

    let (_, overflow) = U512([0, 0, 0, 0, 0, 0, 2, 1]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 3, 1]));
    assert!(!overflow);

    let (_, overflow) = U512([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
        .overflowing_add(U512([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
    assert!(overflow);

    let (_, overflow) = U512([0, 0, 0, 0, 0, 0, 0, ::std::u64::MAX])
        .overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, ::std::u64::MAX]));
    assert!(overflow);

    let (_, overflow) = U512([0, 0, 0, 0, 0, 0, 0, ::std::u64::MAX])
        .overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, 0]));
    assert!(!overflow);
}

#[test]
fn u256_multi_adds() {
    let (result, _) = U256([0, 0, 0, 0]).overflowing_add(U256([0, 0, 0, 0]));
    assert_eq!(result, U256([0, 0, 0, 0]));

    let (result, _) = U256([0, 0, 0, 1]).overflowing_add(U256([0, 0, 0, 1]));
    assert_eq!(result, U256([0, 0, 0, 2]));

    let (result, overflow) = U256([0, 0, 2, 1]).overflowing_add(U256([0, 0, 3, 1]));
    assert_eq!(result, U256([0, 0, 5, 2]));
    assert!(!overflow);

    let (_, overflow) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
        .overflowing_add(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
    assert!(overflow);

    let (_, overflow) = U256([0, 0, 0, ::std::u64::MAX]).overflowing_add(U256([0, 0, 0, ::std::u64::MAX]));
    assert!(overflow);
}


#[test]
fn u256_multi_subs() {
    let (result, _) = U256([0, 0, 0, 0]).overflowing_sub(U256([0, 0, 0, 0]));
    assert_eq!(result, U256([0, 0, 0, 0]));

    let (result, _) = U256([0, 0, 0, 1]).overflowing_sub(U256([0, 0, 0, 1]));
    assert_eq!(result, U256([0, 0, 0, 0]));

    let (_, overflow) = U256([0, 0, 2, 1]).overflowing_sub(U256([0, 0, 3, 1]));
    assert!(overflow);

    let (result, overflow) =
        U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
            .overflowing_sub(U256([::std::u64::MAX/2, ::std::u64::MAX/2, ::std::u64::MAX/2, ::std::u64::MAX/2]));

    assert!(!overflow);
    assert_eq!(U256([::std::u64::MAX/2+1, ::std::u64::MAX/2+1, ::std::u64::MAX/2+1, ::std::u64::MAX/2+1]), result);

    let (result, overflow) = U256([0, 0, 0, 1]).overflowing_sub(U256([0, 0, 1, 0]));
    assert!(!overflow);
    assert_eq!(U256([0, 0, ::std::u64::MAX, 0]), result);

    let (result, overflow) = U256([0, 0, 0, 1]).overflowing_sub(U256([1, 0, 0, 0]));
    assert!(!overflow);
    assert_eq!(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]), result);
}

#[test]
fn u512_multi_subs() {
    let (result, _) = U512([0, 0, 0, 0, 0, 0, 0, 0]).overflowing_sub(U512([0, 0, 0, 0, 0, 0, 0, 0]));
    assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 0, 0]));

    let (result, _) = U512([10, 9, 8, 7, 6, 5, 4, 3]).overflowing_sub(U512([9, 8, 7, 6, 5, 4, 3, 2]));
    assert_eq!(result, U512([1, 1, 1, 1, 1, 1, 1, 1]));

    let (_, overflow) = U512([10, 9, 8, 7, 6, 5, 4, 3]).overflowing_sub(U512([9, 8, 7, 6, 5, 4, 3, 2]));
    assert!(!overflow);

    let (_, overflow) = U512([9, 8, 7, 6, 5, 4, 3, 2]).overflowing_sub(U512([10, 9, 8, 7, 6, 5, 4, 3]));
    assert!(overflow);
}

#[test]
fn u256_multi_carry_all() {
    let (result, _) = U256([::std::u64::MAX, 0, 0, 0]).overflowing_mul(U256([::std::u64::MAX, 0, 0, 0]));
    assert_eq!(U256([1, ::std::u64::MAX-1, 0, 0]), result);

    let (result, _) = U256([0, ::std::u64::MAX, 0, 0]).overflowing_mul(U256([::std::u64::MAX, 0, 0, 0]));
    assert_eq!(U256([0, 1, ::std::u64::MAX-1, 0]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]).overflowing_mul(U256([::std::u64::MAX, 0, 0, 0]));
    assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX-1, 0]), result);

    let (result, _) = U256([::std::u64::MAX, 0, 0, 0]).overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
    assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX-1, 0]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0])
        .overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
    assert_eq!(U256([1, 0, ::std::u64::MAX-1, ::std::u64::MAX]), result);

    let (result, _) = U256([::std::u64::MAX, 0, 0, 0]).overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
    assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]).overflowing_mul(U256([::std::u64::MAX, 0, 0, 0]));
    assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1]), result);

    let (result, _) = U256([::std::u64::MAX, 0, 0, 0]).overflowing_mul(
        U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
    assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
        .overflowing_mul(U256([::std::u64::MAX, 0, 0, 0]));
    assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0])
        .overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
    assert_eq!(U256([1, 0, ::std::u64::MAX, ::std::u64::MAX-1]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0])
        .overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
    assert_eq!(U256([1, 0, ::std::u64::MAX, ::std::u64::MAX-1]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
        .overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
    assert_eq!(U256([1, 0, ::std::u64::MAX, ::std::u64::MAX]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0])
        .overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
    assert_eq!(U256([1, 0, ::std::u64::MAX, ::std::u64::MAX]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0])
        .overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
    assert_eq!(U256([1, 0, 0, ::std::u64::MAX-1]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0])
        .overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
    assert_eq!(U256([1, 0, 0, ::std::u64::MAX]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
        .overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
    assert_eq!(U256([1, 0, 0, ::std::u64::MAX]), result);

    let (result, _) = U256([0, 0, 0, ::std::u64::MAX]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX]));
    assert_eq!(U256([0, 0, 0, 0]), result);

    let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX]));
    assert_eq!(U256([0, 0, 0, ::std::u64::MAX]), result);

    let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
        .overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
    assert_eq!(U256([1, 0, 0, 0]), result);
}

#[test]
fn u256_multi_muls() {
    let (result, _) = U256([0, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, 0]));
    assert_eq!(U256([0, 0, 0, 0]), result);

    let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([1, 0, 0, 0]));
    assert_eq!(U256([1, 0, 0, 0]), result);

    let (result, _) = U256([5, 0, 0, 0]).overflowing_mul(U256([5, 0, 0, 0]));
    assert_eq!(U256([25, 0, 0, 0]), result);

    let (result, _) = U256([0, 5, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
    assert_eq!(U256([0, 0, 25, 0]), result);

    let (result, _) = U256([0, 0, 0, 1]).overflowing_mul(U256([1, 0, 0, 0]));
    assert_eq!(U256([0, 0, 0, 1]), result);

    let (result, _) = U256([0, 0, 0, 5]).overflowing_mul(U256([2, 0, 0, 0]));
    assert_eq!(U256([0, 0, 0, 10]), result);

    let (result, _) = U256([0, 0, 1, 0]).overflowing_mul(U256([0, 5, 0, 0]));
    assert_eq!(U256([0, 0, 0, 5]), result);

    let (result, _) = U256([0, 0, 8, 0]).overflowing_mul(U256([0, 0, 7, 0]));
    assert_eq!(U256([0, 0, 0, 0]), result);

    let (result, _) = U256([2, 0, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
    assert_eq!(U256([0, 10, 0, 0]), result);

    let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX]));
    assert_eq!(U256([0, 0, 0, ::std::u64::MAX]), result);
}

#[test]
fn u256_multi_muls_overflow() {
    let (_, overflow) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, 0]));
    assert!(!overflow);

    let (_, overflow) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX]));
    assert!(!overflow);

    let (_, overflow) = U256([0, 1, 0, 0]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX]));
    assert!(overflow);

    let (_, overflow) = U256([0, 1, 0, 0]).overflowing_mul(U256([0, 1, 0, 0]));
    assert!(!overflow);

    let (_, overflow) = U256([0, 1, 0, ::std::u64::MAX]).overflowing_mul(U256([0, 1, 0, ::std::u64::MAX]));
    assert!(overflow);

    let (_, overflow) = U256([0, ::std::u64::MAX, 0, 0]).overflowing_mul(U256([0, ::std::u64::MAX, 0, 0]));
    assert!(!overflow);

    let (_, overflow) = U256([1, 0, 0, 0]).overflowing_mul(U256([10, 0, 0, 0]));
    assert!(!overflow);

    let (_, overflow) = U256([2, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX / 2]));
    assert!(!overflow);

    let (_, overflow) = U256([0, 0, 8, 0]).overflowing_mul(U256([0, 0, 7, 0]));
    assert!(overflow);
}

#[test]
fn big_endian() {
    let source = U256([1, 0, 0, 0]);
    let mut target = vec![0u8; 32];

    assert_eq!(source, U256::from(1));

    source.to_big_endian(&mut target);
    assert_eq!(
    vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
         0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8],
    target);

    let source = U256([512, 0, 0, 0]);
    let mut target = vec![0u8; 32];

    source.to_big_endian(&mut target);
    assert_eq!(
    vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
         0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8],
    target);

    let source = U256([0, 512, 0, 0]);
    let mut target = vec![0u8; 32];

    source.to_big_endian(&mut target);
    assert_eq!(
    vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
         0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
    target);

    let source = U256::from_str("0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20").unwrap();
    source.to_big_endian(&mut target);
    assert_eq!(
    vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11,
         0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20],
    target);
}

#[test]
#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
fn u256_multi_full_mul() {
    let result = U256([0, 0, 0, 0]).full_mul(U256([0, 0, 0, 0]));
    assert_eq!(U512([0, 0, 0, 0, 0, 0, 0, 0]), result);

    let result = U256([1, 0, 0, 0]).full_mul(U256([1, 0, 0, 0]));
    assert_eq!(U512([1, 0, 0, 0, 0, 0, 0, 0]), result);

    let result = U256([5, 0, 0, 0]).full_mul(U256([5, 0, 0, 0]));
    assert_eq!(U512([25, 0, 0, 0, 0, 0, 0, 0]), result);

    let result = U256([0, 5, 0, 0]).full_mul(U256([0, 5, 0, 0]));
    assert_eq!(U512([0, 0, 25, 0, 0, 0, 0, 0]), result);

    let result = U256([0, 0, 0, 4]).full_mul(U256([4, 0, 0, 0]));
    assert_eq!(U512([0, 0, 0, 16, 0, 0, 0, 0]), result);

    let result = U256([0, 0, 0, 5]).full_mul(U256([2, 0, 0, 0]));
    assert_eq!(U512([0, 0, 0, 10, 0, 0, 0, 0]), result);

    let result = U256([0, 0, 2, 0]).full_mul(U256([0, 5, 0, 0]));
    assert_eq!(U512([0, 0, 0, 10, 0, 0, 0, 0]), result);

    let result = U256([0, 3, 0, 0]).full_mul(U256([0, 0, 3, 0]));
    assert_eq!(U512([0, 0, 0, 9, 0, 0, 0, 0]), result);

    let result = U256([0, 0, 8, 0]).full_mul(U256([0, 0, 6, 0]));
    assert_eq!(U512([0, 0, 0, 0, 48, 0, 0, 0]), result);

    let result = U256([9, 0, 0, 0]).full_mul(U256([0, 3, 0, 0]));
    assert_eq!(U512([0, 27, 0, 0, 0, 0, 0, 0]), result);

    let result = U256([::std::u64::MAX, 0, 0, 0]).full_mul(U256([::std::u64::MAX, 0, 0, 0]));
    assert_eq!(U512([1, ::std::u64::MAX-1, 0, 0, 0, 0, 0, 0]), result);

    let result = U256([0, ::std::u64::MAX, 0, 0]).full_mul(U256([::std::u64::MAX, 0, 0, 0]));
    assert_eq!(U512([0, 1, ::std::u64::MAX-1, 0, 0, 0, 0, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]).full_mul(U256([::std::u64::MAX, 0, 0, 0]));
    assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0, 0, 0]), result);

    let result = U256([::std::u64::MAX, 0, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
    assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0, 0, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
    assert_eq!(U512([1, 0, ::std::u64::MAX-1, ::std::u64::MAX, 0, 0, 0, 0]), result);

    let result = U256([::std::u64::MAX, 0, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
    assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]).full_mul(U256([::std::u64::MAX, 0, 0, 0]));
    assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0, 0]), result);

    let result = U256([::std::u64::MAX, 0, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
    assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]).full_mul(U256([::std::u64::MAX, 0, 0, 0]));
    assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
    assert_eq!(U512([1, 0, ::std::u64::MAX, ::std::u64::MAX-1, ::std::u64::MAX, 0, 0, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
    assert_eq!(U512([1, 0, ::std::u64::MAX, ::std::u64::MAX-1, ::std::u64::MAX, 0, 0, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
    assert_eq!(U512([1, 0, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, ::std::u64::MAX, 0, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
    assert_eq!(U512([1, 0, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, ::std::u64::MAX, 0, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
    assert_eq!(U512([1, 0, 0, ::std::u64::MAX-1, ::std::u64::MAX, ::std::u64::MAX, 0, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
    assert_eq!(U512([1, 0, 0, ::std::u64::MAX,  ::std::u64::MAX-1, ::std::u64::MAX, ::std::u64::MAX, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
    assert_eq!(U512([1, 0, 0, ::std::u64::MAX,  ::std::u64::MAX-1, ::std::u64::MAX, ::std::u64::MAX, 0]), result);

    let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
    assert_eq!(U512([1, 0, 0, 0, ::std::u64::MAX-1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]), result);

    let result = U256([0, 0, 0, ::std::u64::MAX]).full_mul(U256([0, 0, 0, ::std::u64::MAX]));
    assert_eq!(U512([0, 0, 0, 0, 0, 0, 1, ::std::u64::MAX-1]), result);

    let result = U256([1, 0, 0, 0]).full_mul(U256([0, 0, 0, ::std::u64::MAX]));
    assert_eq!(U512([0, 0, 0, ::std::u64::MAX, 0, 0, 0, 0]), result);

    let result = U256([1, 2, 3, 4]).full_mul(U256([5, 0, 0, 0]));
    assert_eq!(U512([5, 10, 15, 20, 0, 0, 0, 0]), result);

    let result = U256([1, 2, 3, 4]).full_mul(U256([0, 6, 0, 0]));
    assert_eq!(U512([0, 6, 12, 18, 24, 0, 0, 0]), result);

    let result = U256([1, 2, 3, 4]).full_mul(U256([0, 0, 7, 0]));
    assert_eq!(U512([0, 0, 7, 14, 21, 28, 0, 0]), result);

    let result = U256([1, 2, 3, 4]).full_mul(U256([0, 0, 0, 8]));
    assert_eq!(U512([0, 0, 0, 8, 16, 24, 32, 0]), result);

    let result = U256([1, 2, 3, 4]).full_mul(U256([5, 6, 7, 8]));
    assert_eq!(U512([5, 16, 34, 60, 61, 52, 32, 0]), result);
}

#[test]
fn u256_multi_muls2() {

    let (result, _) = U256([0, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, 0]));
    assert_eq!(U256([0, 0, 0, 0]), result);

    let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([1, 0, 0, 0]));
    assert_eq!(U256([1, 0, 0, 0]), result);

    let (result, _) = U256([5, 0, 0, 0]).overflowing_mul(U256([5, 0, 0, 0]));
    assert_eq!(U256([25, 0, 0, 0]), result);

    let (result, _) = U256([0, 5, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
    assert_eq!(U256([0, 0, 25, 0]), result);

    let (result, _) = U256([0, 0, 0, 1]).overflowing_mul(U256([1, 0, 0, 0]));
    assert_eq!(U256([0, 0, 0, 1]), result);

    let (result, _) = U256([0, 0, 0, 5]).overflowing_mul(U256([2, 0, 0, 0]));
    assert_eq!(U256([0, 0, 0, 10]), result);

    let (result, _) = U256([0, 0, 1, 0]).overflowing_mul(U256([0, 5, 0, 0]));
    assert_eq!(U256([0, 0, 0, 5]), result);

    let (result, _) = U256([0, 0, 8, 0]).overflowing_mul(U256([0, 0, 7, 0]));
    assert_eq!(U256([0, 0, 0, 0]), result);

    let (result, _) = U256([2, 0, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
    assert_eq!(U256([0, 10, 0, 0]), result);

    let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, u64::max_value()]));
    assert_eq!(U256([0, 0, 0, u64::max_value()]), result);

    let x1: U256 = "0000000000000000000000000000000000000000000000000000012365124623".into();
    let x2sqr_right: U256 = "000000000000000000000000000000000000000000014baeef72e0378e2328c9".into();
    let x1sqr = x1 * x1;
    assert_eq!(x2sqr_right, x1sqr);

    let x1cube = x1sqr * x1;
    let x1cube_right: U256 = "0000000000000000000000000000000001798acde139361466f712813717897b".into();
    assert_eq!(x1cube_right, x1cube);

    let x1quad = x1cube * x1;
    let x1quad_right: U256 = "000000000000000000000001adbdd6bd6ff027485484b97f8a6a4c7129756dd1".into();
    assert_eq!(x1quad_right, x1quad);

    let x1penta = x1quad * x1;
    let x1penta_right: U256 = "00000000000001e92875ac24be246e1c57e0507e8c46cc8d233b77f6f4c72993".into();
    assert_eq!(x1penta_right, x1penta);

    let x1septima = x1penta * x1;
    let x1septima_right: U256 = "00022cca1da3f6e5722b7d3cc5bbfb486465ebc5a708dd293042f932d7eee119".into();
    assert_eq!(x1septima_right, x1septima);
}

#[test]
fn example() {
    let mut val: U256 = 1023.into();
    for _ in 0..200 { val = val * 2.into() }
    assert_eq!(&format!("{}", val), "1643897619276947051879427220465009342380213662639797070513307648");
}

#[test]
fn little_endian() {
    let number: U256 = "00022cca1da3f6e5722b7d3cc5bbfb486465ebc5a708dd293042f932d7eee119".into();
    let expected = [
        0x19, 0xe1, 0xee, 0xd7,
        0x32, 0xf9, 0x42, 0x30,
        0x29, 0xdd, 0x08, 0xa7,
        0xc5, 0xeb, 0x65, 0x64,
        0x48, 0xfb, 0xbb, 0xc5,
        0x3c, 0x7d, 0x2b, 0x72,
        0xe5, 0xf6, 0xa3, 0x1d,
        0xca, 0x2c, 0x02, 0x00
    ];
    let mut result = [0u8; 32];
    number.to_little_endian(&mut result);
    assert_eq!(expected, result);
}