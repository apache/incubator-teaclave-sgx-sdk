// Copyright 2016 bluss
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// General matrix multiply kernel
pub trait GemmKernel {
    type Elem: Element;

    /// align inputs to this
    fn align_to() -> usize;

    /// Kernel rows
    fn mr() -> usize;
    /// Kernel cols
    fn nr() -> usize;

    /// Whether to always use the masked wrapper around the kernel.
    ///
    /// If masked, the kernel is always called with α=1, β=0
    fn always_masked() -> bool;

    fn nc() -> usize;
    fn kc() -> usize;
    fn mc() -> usize;

    /// Matrix multiplication kernel
    ///
    /// This does the matrix multiplication:
    ///
    /// C := alpha * A * B + beta * C
    ///
    /// + `k`: length of data in a, b
    /// + a, b are packed
    /// + c has general strides
    /// + rsc: row stride of c
    /// + csc: col stride of c
    /// + if `beta` is `0.`, then c does not need to be initialized
    unsafe fn kernel(
        k: usize,
        alpha: Self::Elem,
        a: *const Self::Elem,
        b: *const Self::Elem,
        beta: Self::Elem,
        c: *mut Self::Elem, rsc: isize, csc: isize);
}

pub trait Element : Copy {
    fn zero() -> Self;
    fn one() -> Self;
    fn is_zero(&self) -> bool;
    fn scale_by(&mut self, x: Self);
    fn scaled_add(&mut self, alpha: Self, a: Self);
}

impl Element for f32 {
    fn zero() -> Self { 0. }
    fn one() -> Self { 1. }
    fn is_zero(&self) -> bool { *self == 0. }
    fn scale_by(&mut self, x: Self) {
        *self *= x;
    }
    fn scaled_add(&mut self, alpha: Self, a: Self) {
        *self += alpha * a;
    }
}

impl Element for f64 {
    fn zero() -> Self { 0. }
    fn one() -> Self { 1. }
    fn is_zero(&self) -> bool { *self == 0. }
    fn scale_by(&mut self, x: Self) {
        *self *= x;
    }
    fn scaled_add(&mut self, alpha: Self, a: Self) {
        *self += alpha * a;
    }
}
