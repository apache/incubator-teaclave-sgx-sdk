// Copyright 2016 bluss
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use kernel::GemmKernel;
use archparam;

pub enum Gemm { }

pub type T = f64;

const MR: usize = 8;
const NR: usize = 4;

macro_rules! loop_m {
    ($i:ident, $e:expr) => { loop8!($i, $e) };
}
macro_rules! loop_n {
    ($j:ident, $e:expr) => { loop4!($j, $e) };
}

impl GemmKernel for Gemm {
    type Elem = T;

    #[inline(always)]
    fn align_to() -> usize { 0 }

    #[inline(always)]
    fn mr() -> usize { MR }
    #[inline(always)]
    fn nr() -> usize { NR }

    #[inline(always)]
    fn always_masked() -> bool { true }

    #[inline(always)]
    fn nc() -> usize { archparam::D_NC }
    #[inline(always)]
    fn kc() -> usize { archparam::D_KC }
    #[inline(always)]
    fn mc() -> usize { archparam::D_MC }

    #[inline(always)]
    unsafe fn kernel(
        k: usize,
        alpha: T,
        a: *const T,
        b: *const T,
        beta: T,
        c: *mut T, rsc: isize, csc: isize) {
        kernel(k, alpha, a, b, beta, c, rsc, csc)
    }
}

/// matrix multiplication kernel
///
/// This does the matrix multiplication:
///
/// C ← α A B + β C
///
/// + k: length of data in a, b
/// + a, b are packed
/// + c has general strides
/// + rsc: row stride of c
/// + csc: col stride of c
/// + if beta is 0, then c does not need to be initialized
#[inline(always)]
pub unsafe fn kernel(k: usize, alpha: T, a: *const T, b: *const T,
                     beta: T, c: *mut T, rsc: isize, csc: isize)
{
    // using `uninitialized` is a workaround for issue https://github.com/bluss/matrixmultiply/issues/9
    let mut ab: [[T; NR]; MR] = ::std::mem::uninitialized();
    let mut a = a;
    let mut b = b;
    debug_assert_eq!(beta, 0.); // always masked
    loop_m!(i, loop_n!(j, ab[i][j] = 0.));

    // Compute matrix multiplication into ab[i][j]
    unroll_by!(4 => k, {
        loop_m!(i, loop_n!(j, ab[i][j] += at(a, i) * at(b, j)));

        a = a.offset(MR as isize);
        b = b.offset(NR as isize);
    });

    macro_rules! c {
        ($i:expr, $j:expr) => (c.offset(rsc * $i as isize + csc * $j as isize));
    }

    // set C = α A B
    loop_m!(i, loop_n!(j, *c![i, j] = alpha * ab[i][j]));
}

#[inline(always)]
unsafe fn at(ptr: *const T, i: usize) -> T {
    *ptr.offset(i as isize)
}

#[test]
fn test_gemm_kernel() {
    let mut a = [1.; 32];
    let mut b = [0.; 16];
    for (i, x) in a.iter_mut().enumerate() {
        *x = i as f64;
    }
    for i in 0..4 {
        b[i + i * 4] = 1.;
    }
    let mut c = [0.; 32];
    unsafe {
        kernel(4, 1., &a[0], &b[0],
               0., &mut c[0], 1, 8);
        // transposed C so that results line up
    }
    assert_eq!(&a, &c);
}

