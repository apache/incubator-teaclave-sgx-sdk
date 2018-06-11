// Copyright (c) 2016 rust-threshold-secret-sharing developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Various number theoretic utility functions used in the library.

use std::vec::Vec;

/// Euclidean GCD implementation (recursive). The first member of the returned
/// triplet is the GCD of `a` and `b`.
pub fn gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let n = a / b;
        let c = a % b;
        let r = gcd(b, c);
        (r.0, r.2, r.1 - r.2 * n)
    }
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(12, 16), (4, -1, 1));
}


/// Inverse of `k` in the *Zp* field defined by `prime`.
pub fn mod_inverse(k: i64, prime: i64) -> i64 {
    let k2 = k % prime;
    let r = if k2 < 0 {
        -gcd(prime, -k2).2
    } else {
        gcd(prime, k2).2
    };
    (prime + r) % prime
}

#[test]
fn test_mod_inverse() {
    assert_eq!(mod_inverse(3, 7), 5);
}


/// `x` to the power of `e` in the *Zp* field defined by `prime`.
pub fn mod_pow(mut x: i64, mut e: u32, prime: i64) -> i64 {
    let mut acc = 1;
    while e > 0 {
        if e % 2 == 0 {
            // even
            // no-op
        } else {
            // odd
            acc = (acc * x) % prime;
        }
        x = (x * x) % prime; // waste one of these by having it here but code is simpler (tiny bit)
        e = e >> 1;
    }
    acc
}

#[test]
fn test_mod_pow() {
    assert_eq!(mod_pow(2, 0, 17), 1);
    assert_eq!(mod_pow(2, 3, 17), 8);
    assert_eq!(mod_pow(2, 6, 17), 13);

    assert_eq!(mod_pow(-3, 0, 17), 1);
    assert_eq!(mod_pow(-3, 1, 17), -3);
    assert_eq!(mod_pow(-3, 15, 17), -6);
}


/// Compute the 2-radix FFT of `a_coef` in the *Zp* field defined by `prime`.
///
/// `omega` must be a `n`-th principal root of unity,
/// where `n` is the lenght of `a_coef` as well as a power of 2.
/// The result will contains the same number of elements.
#[allow(dead_code)]
pub fn fft2(a_coef: &[i64], omega: i64, prime: i64) -> Vec<i64> {
    use fields::Field;
    let zp = ::fields::montgomery::MontgomeryField32::new(prime as u32);

    let mut data = a_coef.iter().map(|&a| zp.from_i64(a)).collect::<Vec<_>>();
    ::fields::fft::fft2(&zp, &mut *data, zp.from_i64(omega));
    data.iter().map(|a| zp.to_i64(*a)).collect()
}

/// Inverse FFT for `fft2`.
pub fn fft2_inverse(a_point: &[i64], omega: i64, prime: i64) -> Vec<i64> {
    use fields::Field;
    let zp = ::fields::montgomery::MontgomeryField32::new(prime as u32);

    let mut data = a_point.iter().map(|&a| zp.from_i64(a)).collect::<Vec<_>>();
    ::fields::fft::fft2_inverse(&zp, &mut *data, zp.from_i64(omega));
    data.iter().map(|a| zp.to_i64(*a)).collect()
}

#[test]
fn test_fft2() {
    // field is Z_433 in which 354 is an 8th root of unity
    let prime = 433;
    let omega = 354;

    let a_coef = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let a_point = fft2(&a_coef, omega, prime);
    assert_eq!(positivise(&a_point, prime),
               positivise(&[36, -130, -287, 3, -4, 422, 279, -311], prime))
}

#[test]
fn test_fft2_inverse() {
    // field is Z_433 in which 354 is an 8th root of unity
    let prime = 433;
    let omega = 354;

    let a_point = vec![36, -130, -287, 3, -4, 422, 279, -311];
    let a_coef = fft2_inverse(&a_point, omega, prime);
    assert_eq!(positivise(&a_coef, prime), vec![1, 2, 3, 4, 5, 6, 7, 8])
}

/// Compute the 3-radix FFT of `a_coef` in the *Zp* field defined by `prime`.
///
/// `omega` must be a `n`-th principal root of unity,
/// where `n` is the lenght of `a_coef` as well as a power of 3.
/// The result will contains the same number of elements.
pub fn fft3(a_coef: &[i64], omega: i64, prime: i64) -> Vec<i64> {
    use fields::Field;
    let zp = ::fields::montgomery::MontgomeryField32::new(prime as u32);

    let mut data = a_coef.iter().map(|&a| zp.from_i64(a)).collect::<Vec<_>>();
    ::fields::fft::fft3(&zp, &mut *data, zp.from_i64(omega));
    data.iter().map(|a| zp.to_i64(*a)).collect()
}

/// Inverse FFT for `fft3`.
#[allow(dead_code)]
pub fn fft3_inverse(a_point: &[i64], omega: i64, prime: i64) -> Vec<i64> {
    use fields::Field;
    let zp = ::fields::montgomery::MontgomeryField32::new(prime as u32);

    let mut data = a_point.iter().map(|&a| zp.from_i64(a)).collect::<Vec<_>>();
    ::fields::fft::fft3_inverse(&zp, &mut *data, zp.from_i64(omega));
    data.iter().map(|a| zp.to_i64(*a)).collect()
}

#[test]
fn test_fft3() {
    // field is Z_433 in which 150 is an 9th root of unity
    let prime = 433;
    let omega = 150;

    let a_coef = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let a_point = positivise(&fft3(&a_coef, omega, prime), prime);
    assert_eq!(a_point, vec![45, 404, 407, 266, 377, 47, 158, 17, 20])
}

#[test]
fn test_fft3_inverse() {
    // field is Z_433 in which 150 is an 9th root of unity
    let prime = 433;
    let omega = 150;

    let a_point = vec![45, 404, 407, 266, 377, 47, 158, 17, 20];
    let a_coef = positivise(&fft3_inverse(&a_point, omega, prime), prime);
    assert_eq!(a_coef, vec![1, 2, 3, 4, 5, 6, 7, 8, 9])
}

/// Performs a Lagrange interpolation in field Zp at the origin
/// for a polynomial defined by `points` and `values`.
///
/// `points` and `values` are expected to be two arrays of the same size, containing
/// respectively the evaluation points (x) and the value of the polynomial at those point (p(x)).
///
/// The result is the value of the polynomial at x=0. It is also its zero-degree coefficient.
///
/// This is obviously less general than `newton_interpolation_general` as we
/// only get a single value, but it is much faster.
pub fn lagrange_interpolation_at_zero(points: &[i64], values: &[i64], prime: i64) -> i64 {
    assert_eq!(points.len(), values.len());
    // Lagrange interpolation for point 0
    let mut acc = 0i64;
    for i in 0..values.len() {
        let xi = points[i];
        let yi = values[i];
        let mut num = 1i64;
        let mut denum = 1i64;
        for j in 0..values.len() {
            if j != i {
                let xj = points[j];
                num = (num * xj) % prime;
                denum = (denum * (xj - xi)) % prime;
            }
        }
        acc = (acc + yi * num * mod_inverse(denum, prime)) % prime;
    }
    acc
}

/// Holds together points and Newton-interpolated coefficients for fast evaluation.
pub struct NewtonPolynomial<'a> {
    points: &'a [i64],
    coefficients: Vec<i64>,
}


/// General case for Newton interpolation in field Zp.
///
/// Given enough `points` (x) and `values` (p(x)), find the coefficients for `p`.
pub fn newton_interpolation_general<'a>(points: &'a [i64],
                                        values: &[i64],
                                        prime: i64)
                                        -> NewtonPolynomial<'a> {
    let coefficients = compute_newton_coefficients(points, values, prime);
    NewtonPolynomial {
        points: points,
        coefficients: coefficients,
    }
}

#[test]
fn test_newton_interpolation_general() {
    let prime = 17;

    let poly = [1, 2, 3, 4];
    let points = vec![5, 6, 7, 8, 9];
    let values: Vec<i64> =
        points.iter().map(|&point| mod_evaluate_polynomial(&poly, point, prime)).collect();
    assert_eq!(values, vec![8, 16, 4, 13, 16]);

    let recovered_poly = newton_interpolation_general(&points, &values, prime);
    let recovered_values: Vec<i64> =
        points.iter().map(|&point| newton_evaluate(&recovered_poly, point, prime)).collect();
    assert_eq!(recovered_values, values);

    assert_eq!(newton_evaluate(&recovered_poly, 10, prime), 3);
    assert_eq!(newton_evaluate(&recovered_poly, 11, prime), -2);
    assert_eq!(newton_evaluate(&recovered_poly, 12, prime), 8);
}

pub fn newton_evaluate(poly: &NewtonPolynomial, point: i64, prime: i64) -> i64 {
    // compute Newton points
    let mut newton_points = vec![1];
    for i in 0..poly.points.len() - 1 {
        let diff = (point - poly.points[i]) % prime;
        let product = (newton_points[i] * diff) % prime;
        newton_points.push(product);
    }
    let ref newton_coefs = poly.coefficients;
    // sum up
    newton_coefs.iter()
        .zip(newton_points)
        .map(|(coef, point)| (coef * point) % prime)
        .fold(0, |a, b| (a + b) % prime)
}

fn compute_newton_coefficients(points: &[i64], values: &[i64], prime: i64) -> Vec<i64> {
    assert_eq!(points.len(), values.len());

    let mut store: Vec<(usize, usize, i64)> =
        values.iter().enumerate().map(|(index, &value)| (index, index, value)).collect();

    for j in 1..store.len() {
        for i in (j..store.len()).rev() {
            let index_lower = store[i - 1].0;
            let index_upper = store[i].1;

            let point_lower = points[index_lower];
            let point_upper = points[index_upper];
            let point_diff = (point_upper - point_lower) % prime;
            let point_diff_inverse = mod_inverse(point_diff, prime);

            let coef_lower = store[i - 1].2;
            let coef_upper = store[i].2;
            let coef_diff = (coef_upper - coef_lower) % prime;

            let fraction = (coef_diff * point_diff_inverse) % prime;

            store[i] = (index_lower, index_upper, fraction);
        }
    }

    store.iter().map(|&(_, _, v)| v).collect()
}

#[test]
fn test_compute_newton_coefficients() {
    let points = vec![5, 6, 7, 8, 9];
    let values = vec![8, 16, 4, 13, 16];
    let prime = 17;

    let coefficients = compute_newton_coefficients(&points, &values, prime);
    assert_eq!(coefficients, vec![8, 8, -10, 4, 0]);
}

/// Map `values` from `[-n/2, n/2)` to `[0, n)`.
pub fn positivise(values: &[i64], n: i64) -> Vec<i64> {
    values.iter()
        .map(|&value| if value < 0 { value + n } else { value })
        .collect()
}

// deprecated
// fn mod_evaluate_polynomial_naive(coefficients: &[i64], point: i64, prime: i64) -> i64 {
//     // evaluate naively
//     coefficients.iter()
//        .enumerate()
//        .map(|(deg, coef)| (coef * mod_pow(point, deg as u32, prime)) % prime)
//        .fold(0, |a, b| (a + b) % prime)
// }
//
// #[test]
// fn test_mod_evaluate_polynomial_naive() {
//     let poly = vec![1,2,3,4,5,6];
//     let point = 5;
//     let prime = 17;
//     assert_eq!(mod_evaluate_polynomial_naive(&poly, point, prime), 4);
// }

/// Evaluate polynomial given by `coefficients` at `point` in Zp using Horner's method.
pub fn mod_evaluate_polynomial(coefficients: &[i64], point: i64, prime: i64) -> i64 {
    // evaluate using Horner's rule
    //  - to combine with fold we consider the coefficients in reverse order
    let mut reversed_coefficients = coefficients.iter().rev();
    // manually split due to fold insisting on an initial value
    let head = *reversed_coefficients.next().unwrap();
    let tail = reversed_coefficients;
    tail.fold(head, |partial, coef| (partial * point + coef) % prime)
}

#[test]
fn test_mod_evaluate_polynomial() {
    let poly = vec![1, 2, 3, 4, 5, 6];
    let point = 5;
    let prime = 17;
    assert_eq!(mod_evaluate_polynomial(&poly, point, prime), 4);
}
