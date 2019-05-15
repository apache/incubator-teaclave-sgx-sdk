
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(feature = "std")]
extern crate heapsize;

#[cfg(not(feature = "std"))]
extern crate alloc;

// Re-export libcore using an alias so that the macros can work without
// requiring `extern crate core` downstream.
#[doc(hidden)]
pub extern crate core as core_;

use core_::{
	cmp::Ordering,
	hash::{Hash, Hasher},
	fmt,
	ops::Deref,
};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use heapsize::HeapSizeOf;

#[macro_export]
macro_rules! impl_elastic_array {
	($name: ident, $dummy: ident, $size: expr) => (
		#[doc(hidden)]
		enum $dummy<T> {
			Arr([T; $size]),
			Vec(Vec<T>)
		}

		impl<T> $dummy<T> {
			#[doc(hidden)]
			pub fn slice(&self) -> &[T] {
				match *self {
					$dummy::Arr(ref v) => v,
					$dummy::Vec(ref v) => v
				}
			}
		}

		impl<T> Clone for $dummy<T> where T: Copy {
			fn clone(&self) -> $dummy<T> {
				match *self {
					$dummy::Arr(ref a) => $dummy::Arr(*a),
					$dummy::Vec(ref v) => $dummy::Vec(v.clone()),
				}
			}
		}

		pub struct $name<T> {
			raw: $dummy<T>,
			len: usize
		}

		impl<T> Eq for $name<T> where T: Eq { }

		impl<T> fmt::Debug for $name<T> where T: fmt::Debug {
			fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
				match self.raw {
					$dummy::Arr(ref a) => (&a[..self.len]).fmt(f),
					$dummy::Vec(ref v) => v.fmt(f),
				}
			}
		}

		impl<T, U> PartialEq<U> for $name<T> where T: PartialEq, U: Deref<Target=[T]> {
			fn eq(&self, other: &U) -> bool {
				self.slice() == &**other
			}
		}

		impl<T, U> PartialOrd<U> for $name<T> where T: PartialOrd, U: Deref<Target=[T]> {
			fn partial_cmp(&self, other: &U) -> Option<Ordering> {
				(&**self).partial_cmp(&*other)
			}
		}

		impl<T> Ord for $name<T> where T: Ord {
			fn cmp(&self, other: &Self) -> Ordering {
				(&**self).cmp(&*other)
			}
		} 

		impl<T> Hash for $name<T> where T: Hash {
			fn hash<H>(&self, state: &mut H) where H: Hasher {
				self.slice().hash(state)
			}
		}

		#[cfg(feature = "std")]
		impl<T> HeapSizeOf for $name<T> where T: HeapSizeOf {
			fn heap_size_of_children(&self) -> usize {
				match self.raw {
					$dummy::Arr(_) => 0,
					$dummy::Vec(ref v) => v.heap_size_of_children()
				}
			}
		}

		impl<T> Clone for $name<T> where T: Copy {
			fn clone(&self) -> $name<T> {
				$name {
					raw: self.raw.clone(),
					len: self.len,
				}
			}
		}

		impl<T> Default for $name<T> where T: Copy {
			fn default() -> $name<T> {
				Self::new()
			}
		}

		impl<T> $name<T> where T: Copy {
			pub fn new() -> $name<T> {
				$name {
					raw: $dummy::Arr(unsafe { $crate::core_::mem::uninitialized() }),
					len: 0
				}
			}

			pub fn from_slice(slice: &[T]) -> $name<T> {
				let mut v = $name::new();
				v.append_slice(slice);
				v
			}

			pub fn from_vec(vec: Vec<T>) -> $name<T> {
				$name {
					len: vec.len(),
					raw: $dummy::Vec(vec),
				}
			}

			pub fn push(&mut self, e: T) {
				match self.raw {
					$dummy::Arr(ref mut a) if self.len < a.len() => {
						unsafe {
							*a.get_unchecked_mut(self.len) = e;
						}
					},
					$dummy::Arr(_) => {
						let mut vec = Vec::new();
						vec.reserve(self.len + 1);

						unsafe {
							$crate::core_::ptr::copy(self.raw.slice().as_ptr(), vec.as_mut_ptr(), self.len);
							vec.set_len(self.len);
						}

						vec.push(e);
						self.raw = $dummy::Vec(vec);
					},
					$dummy::Vec(ref mut v) => v.push(e)
				}
				self.len += 1;
			}

			pub fn pop(&mut self) -> Option<T> {
				if self.len == 0 {
					return None;
				}

				self.len -= 1;
				match self.raw {
					$dummy::Arr(ref a) => Some(a[self.len]),
					$dummy::Vec(ref mut v) => v.pop()
				}
			}

			pub fn clear(&mut self) {
				self.raw = $dummy::Arr(unsafe { $crate::core_::mem::uninitialized() });
				self.len = 0;
			}

			pub fn append_slice(&mut self, elements: &[T]) {
				let len = self.len;
				self.insert_slice(len, elements)
			}

			pub fn into_vec(self) -> Vec<T> {
				match self.raw {
					$dummy::Arr(a) => {
						let mut vec = Vec::new();
						vec.reserve(self.len);
						unsafe {	
							$crate::core_::ptr::copy(a.as_ptr(), vec.as_mut_ptr(), self.len);
							vec.set_len(self.len);
						}
						vec
					}
					$dummy::Vec(v) => v
				}
			}

			pub fn insert_slice(&mut self, index: usize, elements: &[T]) {
				use $crate::core_::ptr;

				let elen = elements.len();

				if elen == 0 {
					return;
				}
				
				let len = self.len;
				assert!(index <= len);

				match self.raw {
					// it fits in array
					$dummy::Arr(ref mut a) if len + elen <= a.len() => unsafe {
						let p = a.as_mut_ptr().offset(index as isize);
						let ep = elements.as_ptr();

						// shift everything by elen, to make space
						ptr::copy(p, p.offset(elen as isize), len - index);
						// write new elements
						ptr::copy(ep, p, elen);
					},
					// it deosn't, must be rewritten to vec
					$dummy::Arr(_) => unsafe {
						let mut vec = Vec::new();
						vec.reserve(self.len + elen);
						{
							let p = vec.as_mut_ptr();
							let ob = self.raw.slice().as_ptr();
							let ep = elements.as_ptr();
							let oe = ob.offset(index as isize);
							
							// copy begining of an array
							ptr::copy(ob, p, index);

							// copy new elements
							ptr::copy(ep, p.offset(index as isize), elen);

							// copy end of an array	
							ptr::copy(oe, p.offset((index + elen) as isize), len - index);
						}
						vec.set_len(self.len + elen);
						self.raw = $dummy::Vec(vec);
					},
					// just insert it in to vec
					$dummy::Vec(ref mut v) => unsafe {
						v.reserve(elen);

						let p = v.as_mut_ptr().offset(index as isize);
						let ep = elements.as_ptr();

						// shift everything by elen, to make space
						ptr::copy(p, p.offset(elen as isize), len - index);
						// write new elements
						ptr::copy(ep, p, elen);

						v.set_len(self.len + elen);
					}
				}
				self.len += elen;
			}
		}

		impl<T> $name<T> {
			fn slice(&self) -> &[T] {
				match self.raw {
					$dummy::Arr(ref a) => &a[..self.len],
					$dummy::Vec(ref v) => v
				}
			}
		}

		impl<T> Deref for $name<T> {
			type Target = [T];

			#[inline]
			fn deref(&self) -> &[T] {
				self.slice()
			}
		}

		impl<T> $crate::core_::convert::AsRef<[T]> for $name<T> {
			#[inline]
			fn as_ref(&self) -> &[T] {
				self.slice()
			}
		}

		impl<T> $crate::core_::borrow::Borrow<[T]> for $name<T> {
			#[inline]
			fn borrow(&self) -> &[T] {
				self.slice()
			}
		}

		impl<T> $crate::core_::ops::DerefMut for $name<T> {
			#[inline]
			fn deref_mut(&mut self) -> &mut [T] {
				match self.raw {
					$dummy::Arr(ref mut a) => &mut a[..self.len],
					$dummy::Vec(ref mut v) => v
				}
			}
		}

		impl<'a, T: 'a + Copy> From<&'a [T]> for $name<T> {
			fn from(s: &'a [T]) -> Self { Self::from_slice(s) }
		}
	)
}

impl_elastic_array!(ElasticArray2, ElasticArray2Dummy, 2);
impl_elastic_array!(ElasticArray4, ElasticArray4Dummy, 4);
impl_elastic_array!(ElasticArray8, ElasticArray8Dummy, 8);
impl_elastic_array!(ElasticArray16, ElasticArray16Dummy, 16);
impl_elastic_array!(ElasticArray32, ElasticArray32Dummy, 32);
impl_elastic_array!(ElasticArray36, ElasticArray36Dummy, 36);
impl_elastic_array!(ElasticArray64, ElasticArray64Dummy, 64);
impl_elastic_array!(ElasticArray128, ElasticArray128Dummy, 128);
impl_elastic_array!(ElasticArray256, ElasticArray256Dummy, 256);
impl_elastic_array!(ElasticArray512, ElasticArray512Dummy, 512);
impl_elastic_array!(ElasticArray1024, ElasticArray1024Dummy, 1024);
impl_elastic_array!(ElasticArray2048, ElasticArray2048Dummy, 2048);

#[cfg(test)]
mod tests {

	type BytesShort = super::ElasticArray2<u8>;

	#[test]
	fn it_works() {
		let mut bytes = BytesShort::new();
		assert_eq!(bytes.len(), 0);
		bytes.push(1);
		assert_eq!(bytes.len(), 1);
		assert_eq!(bytes[0], 1);
		bytes.push(2);
		assert_eq!(bytes[1], 2);
		assert_eq!(bytes.len(), 2);
		bytes.push(3);
		assert_eq!(bytes[2], 3);
		assert_eq!(bytes.len(), 3);
		assert_eq!(bytes.pop(), Some(3));
		assert_eq!(bytes.len(), 2);
		assert_eq!(bytes.pop(), Some(2));
		assert_eq!(bytes.pop(), Some(1));
		assert_eq!(bytes.pop(), None);
	}

	#[test]
	fn test_insert_slice() {
		let mut bytes = BytesShort::new();
		bytes.push(1);
		bytes.push(2);
		bytes.insert_slice(1, &[3, 4]);
		assert_eq!(bytes.len(), 4);
		let r: &[u8] = &bytes;
		assert_eq!(r, &[1, 3, 4, 2]);
	}

	#[test]
	fn append_slice() {
		let mut bytes = BytesShort::new();
		bytes.push(1);
		bytes.append_slice(&[3, 4]);
		let r: &[u8] = &bytes;
		assert_eq!(r.len(), 3);
		assert_eq!(r, &[1, 3 ,4]);
	}

	#[test]
	fn use_in_map() {
		#[cfg(feature = "std")]
		use std::collections::BTreeMap;
		#[cfg(not(feature = "std"))]
		use alloc::collections::BTreeMap;
		use ::core_::borrow::Borrow;
		let mut map: BTreeMap<BytesShort, i32> = Default::default();
		let mut bytes = BytesShort::new();
		bytes.append_slice(&[3, 4]);
		assert_eq!(bytes.borrow() as &[u8], &[3, 4][..]);
		map.insert(bytes, 1);
		assert_eq!(map.get(&[3, 4][..]), Some(&1i32));
	}

}
