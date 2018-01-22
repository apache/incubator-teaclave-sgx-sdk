extern crate matrixmultiply;
use matrixmultiply::{sgemm, dgemm};

use std::fmt::{Display, Debug};

trait Float : Copy + Display + Debug + PartialEq {
    fn zero() -> Self;
    fn one() -> Self;
    fn from(i64) -> Self;
    fn nan() -> Self;
}

impl Float for f32 {
    fn zero() -> Self { 0. }
    fn one() -> Self { 1. }
    fn from(x: i64) -> Self { x as Self }
    fn nan() -> Self { 0./0. }
}

impl Float for f64 {
    fn zero() -> Self { 0. }
    fn one() -> Self { 1. }
    fn from(x: i64) -> Self { x as Self }
    fn nan() -> Self { 0./0. }
}


trait Gemm : Sized {
    unsafe fn gemm(
        m: usize, k: usize, n: usize,
        alpha: Self,
        a: *const Self, rsa: isize, csa: isize,
        b: *const Self, rsb: isize, csb: isize,
        beta: Self,
        c: *mut Self, rsc: isize, csc: isize);
}

impl Gemm for f32 {
    unsafe fn gemm(
        m: usize, k: usize, n: usize,
        alpha: Self,
        a: *const Self, rsa: isize, csa: isize,
        b: *const Self, rsb: isize, csb: isize,
        beta: Self,
        c: *mut Self, rsc: isize, csc: isize) {
        sgemm(
            m, k, n,
            alpha,
            a, rsa, csa,
            b, rsb, csb,
            beta,
            c, rsc, csc)
    }
}

impl Gemm for f64 {
    unsafe fn gemm(
        m: usize, k: usize, n: usize,
        alpha: Self,
        a: *const Self, rsa: isize, csa: isize,
        b: *const Self, rsb: isize, csb: isize,
        beta: Self,
        c: *mut Self, rsc: isize, csc: isize) {
        dgemm(
            m, k, n,
            alpha,
            a, rsa, csa,
            b, rsb, csb,
            beta,
            c, rsc, csc)
    }
}

#[test]
fn test_sgemm() {
    test_gemm::<f32>();
}
#[test]
fn test_dgemm() {
    test_gemm::<f64>();
}

fn test_gemm<F>() where F: Gemm + Float {
    test_mul_with_id::<F>(4, 4, true);
    test_mul_with_id::<F>(8, 8, true);
    test_mul_with_id::<F>(32, 32, false);
    test_mul_with_id::<F>(128, 128, false);
    test_mul_with_id::<F>(17, 128, false);
    for i in 0..12 {
        for j in 0..12 {
            test_mul_with_id::<F>(i, j, true);
        }
    }
    /*
    */
    test_mul_with_id::<F>(17, 257, false);
    test_mul_with_id::<F>(24, 512, false);
    for i in 0..10 {
        for j in 0..10 {
            test_mul_with_id::<F>(i * 4, j * 4, true);
        }
    }
    test_mul_with_id::<F>(266, 265, false);
    test_mul_id_with::<F>(4, 4, true);
    for i in 0..12 {
        for j in 0..12 {
            test_mul_id_with::<F>(i, j, true);
        }
    }
    test_mul_id_with::<F>(266, 265, false);
    test_scale::<F>(4, 4, 4, true);
    test_scale::<F>(19, 20, 16, true);
    test_scale::<F>(150, 140, 128, false);
}

/// multiply a M x N matrix with an N x N id matrix
#[cfg(test)]
fn test_mul_with_id<F>(m: usize, n: usize, small: bool)
    where F: Gemm + Float
{
    let (m, k, n) = (m, n, n);
    let mut a = vec![F::zero(); m * k]; 
    let mut b = vec![F::zero(); k * n];
    let mut c = vec![F::zero(); m * n];
    println!("test matrix with id input M={}, N={}", m, n);

    for (i, elt) in a.iter_mut().enumerate() {
        *elt = F::from(i as i64);
    }
    for i in 0..k {
        b[i + i * k] = F::one();
    }

    unsafe {
        F::gemm(
            m, k, n,
            F::one(),
            a.as_ptr(), k as isize, 1,
            b.as_ptr(), n as isize, 1,
            F::zero(),
            c.as_mut_ptr(), n as isize, 1,
            )
    }
    for (i, (x, y)) in a.iter().zip(&c).enumerate() {
        if x != y {
            if k != 0 && n != 0 && small {
                for row in a.chunks(k) {
                    println!("{:?}", row);
                }
                for row in b.chunks(n) {
                    println!("{:?}", row);
                }
                for row in c.chunks(n) {
                    println!("{:?}", row);
                }
            }
            panic!("mismatch at index={}, x: {}, y: {} (matrix input M={}, N={})",
                   i, x, y, m, n);
        }
    }
    println!("passed matrix with id input M={}, N={}", m, n);
}

/// multiply a K x K id matrix with an K x N matrix
#[cfg(test)]
fn test_mul_id_with<F>(k: usize, n: usize, small: bool) 
    where F: Gemm + Float
{
    let (m, k, n) = (k, k, n);
    let mut a = vec![F::zero(); m * k]; 
    let mut b = vec![F::zero(); k * n];
    let mut c = vec![F::zero(); m * n];

    for i in 0..k {
        a[i + i * k] = F::one();
    }
    for (i, elt) in b.iter_mut().enumerate() {
        *elt = F::from(i as i64);
    }

    unsafe {
        F::gemm(
            m, k, n,
            F::one(),
            a.as_ptr(), k as isize, 1,
            b.as_ptr(), n as isize, 1,
            F::zero(),
            c.as_mut_ptr(), n as isize, 1,
            )
    }
    for (i, (x, y)) in b.iter().zip(&c).enumerate() {
        if x != y {
            if k != 0 && n != 0 && small {
                for row in a.chunks(k) {
                    println!("{:?}", row);
                }
                for row in b.chunks(n) {
                    println!("{:?}", row);
                }
                for row in c.chunks(n) {
                    println!("{:?}", row);
                }
            }
            panic!("mismatch at index={}, x: {}, y: {} (matrix input M={}, N={})",
                   i, x, y, m, n);
        }
    }
    println!("passed id with matrix input K={}, N={}", k, n);
}

#[cfg(test)]
fn test_scale<F>(m: usize, k: usize, n: usize, small: bool)
    where F: Gemm + Float
{
    let (m, k, n) = (m, k, n);
    let mut a = vec![F::zero(); m * k]; 
    let mut b = vec![F::zero(); k * n];
    let mut c1 = vec![F::one(); m * n];
    let mut c2 = vec![F::nan(); m * n];
    // init c2 with NaN to test the overwriting behavior when beta = 0.

    for (i, elt) in a.iter_mut().enumerate() {
        *elt = F::from(i as i64);
    }
    for (i, elt) in b.iter_mut().enumerate() {
        *elt = F::from(i as i64);
    }

    unsafe {
        // 4 A B
        F::gemm(
            m, k, n,
            F::from(3),
            a.as_ptr(), k as isize, 1,
            b.as_ptr(), n as isize, 1,
            F::zero(),
            c1.as_mut_ptr(), n as isize, 1,
        );

        // A B 
        F::gemm(
            m, k, n,
            F::one(),
            a.as_ptr(), k as isize, 1,
            b.as_ptr(), n as isize, 1,
            F::zero(),
            c2.as_mut_ptr(), n as isize, 1,
        );
        // (2 A B) + A B
        F::gemm(
            m, k, n,
            F::one(),
            a.as_ptr(), k as isize, 1,
            b.as_ptr(), n as isize, 1,
            F::from(2),
            c2.as_mut_ptr(), n as isize, 1,
        );
    }
    for (i, (x, y)) in c1.iter().zip(&c2).enumerate() {
        if x != y {
            if k != 0 && n != 0 && small {
                for row in a.chunks(k) {
                    println!("{:?}", row);
                }
                for row in b.chunks(n) {
                    println!("{:?}", row);
                }
                for row in c1.chunks(n) {
                    println!("{:?}", row);
                }
                for row in c2.chunks(n) {
                    println!("{:?}", row);
                }
            }
            panic!("mismatch at index={}, x: {}, y: {} (matrix input M={}, N={})",
                   i, x, y, m, n);
        }
    }
    println!("passed matrix with id input M={}, N={}", m, n);
}
