//! Interleave two ordered iterators to create a new ordered iterator.
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]
#[cfg(not(target_env = "sgx"))]
extern crate sgx_tstd as std;

use std::cmp::Ord;
use std::iter::Fuse;

enum Pending<T> {
	None,
	A(T),
	B(T),
}

impl<T> Pending<T> {
	fn take(&mut self) -> Self {
		::std::mem::replace(self, Pending::None)
	}
}

/// Iterator that encapsulates two other ordered iterators to yield their results
/// in order.
pub struct InterleaveOrdered<A, B> where A: Iterator, B: Iterator<Item=A::Item> {
	pending_next: Pending<A::Item>,
	a: Fuse<A>,
	b: Fuse<B>,
}

impl<A, B> Iterator for InterleaveOrdered<A, B>
	where A: Iterator,
		  B: Iterator<Item=A::Item>,
		  A::Item: Ord
{
	type Item = A::Item;

	fn next(&mut self) -> Option<Self::Item> {
		let (a, b) = match self.pending_next.take() {
			Pending::None => (self.a.next(), self.b.next()),
			Pending::A(a) => (Some(a), self.b.next()),
			Pending::B(b) => (self.a.next(), Some(b)),
		};

		match (a, b) {
			(Some(a), Some(b)) => {
				let (res, pending) = if a < b {
					(a, Pending::B(b))
				} else {
					(b, Pending::A(a))
				};

				self.pending_next = pending;
				Some(res)
			}
			(Some(x), _) | (_, Some(x)) => Some(x),
			(None, None) => None
		}
	}
}

/// Interleave two ordered iterators, yielding a new iterator whose items are also ordered.
///
/// ```rust
/// use interleaved_ordered::interleave_ordered;
/// let a = [1, 1, 2, 3, 5, 7, 9];
/// let b = [2, 3, 4, 5, 6, 7, 10];
/// let iter = interleave_ordered(&a, b.iter());

/// assert_eq!(
///    interleave_ordered(&a, &b).cloned().collect::<Vec<_>>(),
///    vec![1, 1, 2, 2, 3, 3, 4, 5, 5, 6, 7, 7, 9, 10]
/// )
/// ```
pub fn interleave_ordered<A, B>(a: A, b: B) -> InterleaveOrdered<A::IntoIter, B::IntoIter>
	where A: IntoIterator, B: IntoIterator<Item=A::Item>
{
	InterleaveOrdered {
		pending_next: Pending::None,
		a: a.into_iter().fuse(),
		b: b.into_iter().fuse(),
	}
}
