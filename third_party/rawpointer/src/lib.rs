// Copyright 2016 bluss and rawpointer developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![no_std]

use core::mem::size_of;

/// Return the number of elements of `T` from `start` to `end`.<br>
/// Return the arithmetic difference if `T` is zero size.
#[inline(always)]
pub fn ptrdistance<T>(start: *const T, end: *const T) -> usize {
    let size = size_of::<T>();
    if size == 0 {
        (end as usize).wrapping_sub(start as usize)
    } else {
        (end as usize - start as usize) / size
    }
}

/// Extension methods for raw pointers
pub trait PointerExt : Copy {
    unsafe fn offset(self, i: isize) -> Self;

    /// Increment the pointer by 1, and return its new value.
    ///
    /// Equivalent to the C idiom `++ptr`.
    #[inline(always)]
    unsafe fn pre_inc(&mut self) -> Self {
        *self = self.offset(1);
        *self
    }

    /// Increment the pointer by 1, but return its old value.
    ///
    /// Equivalent to the C idiom `ptr++`.
    #[inline(always)]
    unsafe fn post_inc(&mut self) -> Self {
        let current = *self;
        *self = self.offset(1);
        current
    }

    /// Decrement the pointer by 1, and return its new value.
    ///
    /// Equivalent to the C idiom `--ptr`.
    #[inline(always)]
    unsafe fn pre_dec(&mut self) -> Self {
        *self = self.offset(-1);
        *self
    }

    /// Decrement the pointer by 1, but return its old value.
    ///
    /// Equivalent to the C idiom `ptr--`.
    #[inline(always)]
    unsafe fn post_dec(&mut self) -> Self {
        let current = *self;
        *self = self.offset(-1);
        current
    }

    /// Increment by 1
    #[inline(always)]
    unsafe fn inc(&mut self) {
        *self = self.offset(1);
    }

    /// Decrement by 1
    #[inline(always)]
    unsafe fn dec(&mut self) {
        *self = self.offset(-1);
    }

    /// Offset the pointer by `s` multiplied by `index`.
    #[inline(always)]
    unsafe fn stride_offset(self, s: isize, index: usize) -> Self {
        self.offset(s * index as isize)
    }
}

impl<T> PointerExt for *const T {
    #[inline(always)]
    unsafe fn offset(self, i: isize) -> Self {
        self.offset(i)
    }
}

impl<T> PointerExt for *mut T {
    #[inline(always)]
    unsafe fn offset(self, i: isize) -> Self {
        self.offset(i)
    }
}


#[cfg(test)]
mod tests {
    use super::PointerExt;

    #[test]
    fn it_works() {
        unsafe {
            let mut xs = [0; 16];
            let mut ptr = xs.as_mut_ptr();
            let end = ptr.offset(4);
            let mut i = 0;
            while ptr != end {
                *ptr.post_inc() = i;
                i += 1;
            }
            assert_eq!(&xs[..8], &[0, 1, 2, 3, 0, 0, 0, 0]);
        }
    }
}
