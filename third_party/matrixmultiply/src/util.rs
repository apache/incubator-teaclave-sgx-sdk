// Copyright 2016 bluss
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cmp::min;

pub struct RangeChunk { i: usize, n: usize, chunk: usize }

/// Create an iterator that splits `n` in chunks of size `chunk`;
/// the last item can be an uneven chunk.
pub fn range_chunk(n: usize, chunk: usize) -> RangeChunk {
    RangeChunk {
        i: 0,
        n: n,
        chunk: chunk,
    }
}

impl Iterator for RangeChunk {
    type Item = (usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.n == 0 {
            None
        } else {
            let i = self.i;
            let rem = min(self.n, self.chunk);
            self.i += 1;
            self.n -= rem;
            Some((i, rem))
        }
    }
}

#[inline]
pub fn round_up_to(x: usize, multiple_of: usize) -> usize {
    let (mut d, r) = (x / multiple_of, x % multiple_of);
    if r > 0 { d += 1; }
    d * multiple_of
}
