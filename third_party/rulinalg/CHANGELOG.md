# Change Log

This document will be used to keep track of changes made between release versions. I'll do my best to note any breaking changes!

## 0.4.2

### Breaking Changes

- None, but `Matrix::lup_decomp` has been deprecated and will be removed soon.

### Features

- Added dedicated `PermutationMatrix`. This type provides more efficent
operations with permutation matrices.
- Added CSV read/write functionality to `Matrix`. Under `io` feature flag.
- Added column iterators, accessed via `BaseMatrix::col_iter` and `BaseMatrixMut::col_iter_mut` functions.
- Added new `PartialPivLU` struct which contains the result of an LUP decomposition.
This struct will replace the `Matrix::lup_decomp` function in a future release.

### Bug Fixes

- Fixed an overflow bug with `SliceIter`.

### Minor Changes

- Fixed (very) minor performance issue in `min`/`max` functions.

## 0.4.1

### Breaking Changes

- None

### Features

- Added new `vector!` macro for `Vector` construction.
- Added new `min` and `max` functions to the `BaseMatrix` trait.
- Implemented conversion traits to convert `Row`/`Column` types to `Vector`.

### Bug Fixes

- None

### Minor Changes

- Performance improvement to `BaseMatrix::sum_rows` function. also
gives passive improvement to `mean` and `variance`.
- Improving layout of the `vector` module.
- Made `matrix!` macro use square brackets consistently.

## 0.4.0

This release includes mostly quality of life changes for users of rulinalg. We do some work to conform more to community
standards with naming, for example `iter_rows` becoming `row_iter`. Additionally several `Matrix` functions now consume `self`
where before they took a reference and immediately cloned `self`.

Another noticable change is the addition of new `Row` and `Column` types. These types are returned by functions which access
single rows or columns in a matrix. With these new types we aim to make it easy for users to do matrix operations on single
rows and columns while maintaining performance where necessary.

This release also welcomes an overhaul of the `Metric` trait. This trait didn't really make sense and only allowed computation
of the euclidean norm. We have created new `Norm` and `Metric` traits for both `Vector` and matrix types (we would like a single
pair of traits but this is not possible without specialization). These new traits allow users to write code which is generic over
norms and specify their own norms. We also provide `Metric` implementations for all `Norm` implementors by computing the norm of
the difference between the objects.

The full release notes are below.

### New Contributors

- [sinhrks](https://github.com/sinhrks)
- [c410-f3r](https://github.com/c410-f3r)
- [Andlon](https://github.com/Andlon)
(Has been involved for a while but I missed him from these lists. Sorry!)

### Breaking Changes

- The `reslice` function for `MatrixSlice` and `MatrixSliceMut` has been
depreciated.
- Rename iterator functions to `*_iter(_mut)`. Affected functions are:
`iter_diag`, `iter_diag_mut`, `iter_rows`, `iter_rows_mut`.
- The `BaseMatrix` `diag` function now returns an iterator.
- Removed the `Metric` trait and all implementations.
- Some functions now consume `self` instead of cloning internally: `eigenvalues`,
`eigendecomp`, `lup_decomp`, `solve`, `inverse` and `det`.
- The `get_row` no longer returns a `&[T]`. Instead it returns the new `Row` type.
- Row iterator no longer has a `&[T]` item. Instead if uses the new `Row` type.
- Moved the `BaseMatrix` and `BaseMatrixMut` traits to a new `matrix/base` module.

### Features

- Implemented `FromIterator` for `Vector`.
- Implemented `from_fn` for `Vector`.
- Implemented `get_unchecked` for `Vector`.
- Implemented `try_into` function using
[num's `NumCast`](http://rust-num.github.io/num/num/cast/trait.NumCast.html) for `Matrix`.
- Added new traits to replace `Metric`; `MatrixNorm` and `VectorNorm`. These come with
`MatrixMetric` and `VectorMetric` traits too.
- Added new `Euclidean` and `Lp` norms.
- The `get_row` functions now return the new `Row` type.
- Added a new `get_col` function which returns the new `Column` type.
- The `row_iter` function uses the new `Row` type as the iterator `Item`.

### Bug Fixes

- Fixed a bug in the ULP comparator where only exact matches were allowed.

### Minor Changes

- The `swap_rows` and `swap_cols` functions are now no-ops if given two identical
indices.
- Splitting out the `slice` module for developer QOL.

## 0.3.7

### New Contributors

- [mabruckner](https://github.com/mabruckner)

### Breaking Changes

- None

### Features

- Added new `assert_matrix_eq!` and `assert_vector_eq!` macros
for easier equality checks. Provides multiple equality comparisons:
`ulp`, `abs`, `float`, `exact`.

### Bug Fixes

- Further improvements (performance and stability) to the LU decomposition algorithm.

### Minor Changes

- Removed import warning on `lu` module.

## 0.3.6

### Breaking Changes

- None

### Features

- None

### Bug Fixes

- Improved numerical stability of the LUP decomposition.

### Minor Changes

- None

## 0.3.5

### New Contributors

- [gcollura](https://github.com/gcollura)

### Breaking Changes

- None

### Features

- Added new `iter_diag` and `iter_diag_mut` functions to `BaseMatrix`
and `BaseMatrixMut` respectively.

### Bug Fixes

- The `matrix!` macro now works on empty matrices.

### Minor Changes

- Some refactoring of `decomposition` module.
- More lenient error handling on triangular solvers.
They no longer `assert!` that a matrix is triangular.
- All tests are now using `matrix!` macro and other
tidier constructors.

## 0.3.4

### New Contributors

- [andrewcsmith](https://github.com/andrewcsmith)
- [nwtnian](https://github.com/nwtnian)

### Breaking Changes

- Removed the `MachineEpsilon` trait. The same functionality
now exists in [num](https://github.com/rust-num/num).

### Features

- Implemented `From`/`Into` for traits for `Vec` and `Vector`.

### Bug Fixes

- `det()` now returns `0` instead of panicking if `Matrix` is singular.

### Minor Changes

- None

## 0.3.3

### New Contributors

- [Andlon](https://github.com/Andlon)
- [regexident](https://github.com/regexident)
- [tokahuke](https://github.com/tokahuke)

### Breaking Changes

- None

### Features

- SVD now returns singular values in descending order.
- Implemented a new `matrix!` macro for creating (small) matrices.
- Added a `from_fn` constructor for `Matrix`.
- Implementing `IndexMut` for `Vector`.
- Added `iter` and `iter_mut` for `Vector`.
- Implemented `IntoIter` for `Vector`.

### Bug Fixes

- Fixed bug with SVD convergence (would loop endlessly).
- Singular values from SVD are now non-negative.

### Minor Changes

- None

## 0.3.2

### New Contributors

- [eugene-bulkin](https://github.com/eugene-bulkin)

### Breaking Changes

- `Matrix::variance` now returns a `Result`.

### Features

- Added `swap_rows` and `swap_cols` function to `BaseMatrixMut`.

### Minor Changes

- Implemented `Display` for `Vector`.

## 0.3.1

### New Contributors

- [scholtzan](https://github.com/scholtzan)
- [theotherphil](https://github.com/theotherphil)

### Breaking Changes

- None

### Features

- None

### Minor Changes

- Improved documentation for `sum_rows` and `sum_cols` functions.
- Generalized signature of `select_rows` and `select_cols`. These functions now
take an `ExactSizeIterator` instead of a slice.

## 0.3.0

This is a large release which refactors most of the `matrix` module.
We modify the `BaseSlice` trait to encompass `Matrix` functionality too - hence
renaming it `BaseMatrix`. The motivation behind this is to allow us to be generic
over `Matrix`/`MatrixSlice`/`MatrixSliceMut`.

### Breaking Changes

- Refactor `BaseSlice` trait as `BaseMatrix`. Implement this trait for `Matrix` too.
- Much of the `Matrix` functionality is now implemented behind the `BaseMatrix` trait. 
It will need to be `use`d to access this functionality.

### Features

- Add a new `BaseMatrixMut` trait for `Matrix` and `MatrixSliceMut`.
- Many methods which were previously for `Matrix` only or for `MatrixSlice(Mut)` only now
work with both!

### Minor Changes

- Fixing a bug in the `sub_slice` method.
- Modifying some unsafe code to use equivalent iterators instead.
- More benchmarks for wider performance regression coverage.

## 0.2.2

### Breaking Changes

-None

### Features

- Vector and Matrix now derive the `Eq` trait.
- Vector and Matrix now derive the `Hash` trait.

### Minor Changes

- None

## 0.2.1

### New Contributors

- [brendan-rius](https://github.com/brendan-rius)
- [tafia](https://github.com/tafia)

### Breaking Changes

- None

### Features

- Adding new `get_row_*` methods for all `Matrix` types. Includes
mutable and unchecked `get` functions.

### Minor Changes

- None

## 0.2.0

### Breaking Changes

- Upper Hessenberg decomposition now consumes the input `Matrix` (instead of cloning at the start).

### Features

- Added Bidiagonal decomposition.
- Added Singular Value Decomposition.

### Minor Changes

- Fixed a bug where `get_unchecked_mut` returned `&T` instead of `&mut T`.

## 0.1.0

This release marks the separation of rulinalg from [rusty-machine](https://github.com/AtheMathmo/rusty-machine).

Rulinalg is now its own crate!