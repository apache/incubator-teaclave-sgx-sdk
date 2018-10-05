# Release 0.2.0

### Enhancements

- [`BigInt` and `BigUint` now implement `Product` and `Sum`][22] for iterators
  of any item that we can `Mul` and `Add`, respectively.  For example, a
  factorial can now be simply: `let f: BigUint = (1u32..1000).product();`
- [`BigInt` now supports two's-complement logic operations][26], namely
  `BitAnd`, `BitOr`, `BitXor`, and `Not`.  These act conceptually as if each
  number had an infinite prefix of `0` or `1` bits for positive or negative.
- [`BigInt` now supports assignment operators][41] like `AddAssign`.
- [`BigInt` and `BigUint` now support conversions with `i128` and `u128`][44],
  if sufficient compiler support is detected.
- [`BigInt` and `BigUint` now implement rand's `SampleUniform` trait][48], and
  [a custom `RandomBits` distribution samples by bit size][49].
- The release also includes other miscellaneous improvements to performance.

### Breaking Changes

- [`num-bigint` now requires rustc 1.15 or greater][23].
- [The crate now has a `std` feature, and won't build without it][46].  This is
  in preparation for someday supporting `#![no_std]` with `alloc`.
- [The `serde` dependency has been updated to 1.0][24], still disabled by
  default.  The `rustc-serialize` crate is no longer supported by `num-bigint`.
- [The `rand` dependency has been updated to 0.5][48], now disabled by default.
  This requires rustc 1.22 or greater for `rand`'s own requirement.
- [`Shr for BigInt` now rounds down][8] rather than toward zero, matching the
  behavior of the primitive integers for negative values.
- [`ParseBigIntError` is now an opaque type][37].
- [The `big_digit` module is no longer public][38], nor are the `BigDigit` and
  `DoubleBigDigit` types and `ZERO_BIG_DIGIT` constant that were re-exported in
  the crate root.  Public APIs which deal in digits, like `BigUint::from_slice`,
  will now always be base-`u32`.

**Contributors**: @clarcharr, @cuviper, @dodomorandi, @tiehuis, @tspiteri

[8]: https://github.com/rust-num/num-bigint/pull/8
[22]: https://github.com/rust-num/num-bigint/pull/22
[23]: https://github.com/rust-num/num-bigint/pull/23
[24]: https://github.com/rust-num/num-bigint/pull/24
[26]: https://github.com/rust-num/num-bigint/pull/26
[37]: https://github.com/rust-num/num-bigint/pull/37
[38]: https://github.com/rust-num/num-bigint/pull/38
[41]: https://github.com/rust-num/num-bigint/pull/41
[44]: https://github.com/rust-num/num-bigint/pull/44
[46]: https://github.com/rust-num/num-bigint/pull/46
[48]: https://github.com/rust-num/num-bigint/pull/48
[49]: https://github.com/rust-num/num-bigint/pull/49

# Release 0.1.44

- [Division with single-digit divisors is now much faster.][42]
- The README now compares [`ramp`, `rug`, `rust-gmp`][20], and [`apint`][21].

**Contributors**: @cuviper, @Robbepop

[20]: https://github.com/rust-num/num-bigint/pull/20
[21]: https://github.com/rust-num/num-bigint/pull/21
[42]: https://github.com/rust-num/num-bigint/pull/42

# Release 0.1.43

- [The new `BigInt::modpow`][18] performs signed modular exponentiation, using
  the existing `BigUint::modpow` and rounding negatives similar to `mod_floor`.

**Contributors**: @cuviper

[18]: https://github.com/rust-num/num-bigint/pull/18


# Release 0.1.42

- [num-bigint now has its own source repository][num-356] at [rust-num/num-bigint][home].
- [`lcm` now avoids creating a large intermediate product][num-350].
- [`gcd` now uses Stein's algorithm][15] with faster shifts instead of division.
- [`rand` support is now extended to 0.4][11] (while still allowing 0.3).

**Contributors**: @cuviper, @Emerentius, @ignatenkobrain, @mhogrefe

[home]: https://github.com/rust-num/num-bigint
[num-350]: https://github.com/rust-num/num/pull/350
[num-356]: https://github.com/rust-num/num/pull/356
[11]: https://github.com/rust-num/num-bigint/pull/11
[15]: https://github.com/rust-num/num-bigint/pull/15


# Prior releases

No prior release notes were kept.  Thanks all the same to the many
contributors that have made this crate what it is!

