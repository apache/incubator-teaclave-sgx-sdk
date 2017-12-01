# rulinalg

[![Join the chat at https://gitter.im/rulinalg/Lobby](https://badges.gitter.im/rulinalg/Lobby.svg)](https://gitter.im/rulinalg/Lobby?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge) [![Build Status](https://travis-ci.org/AtheMathmo/rulinalg.svg?branch=master)](https://travis-ci.org/AtheMathmo/rulinalg)

The crate is currently on [version 0.4.2](https://crates.io/crates/rulinalg).

Read the [API Documentation](https://athemathmo.github.io/rulinalg) to learn more.

---

## Summary

Rulinalg is a linear algebra library written in Rust that doesn't require heavy external dependencies.

The goal of rulinalg is to provide efficient implementations of common linear algebra techniques
in Rust.

Rulinalg was initially a part of [rusty-machine](https://github.com/AtheMathmo/rusty-machine), a machine
learning library in Rust.

#### Contributing

This project is currently [looking for contributors](CONTRIBUTING.md) of all capacities!

---

## Implementation

This project is implemented using [Rust](https://www.rust-lang.org/).

Currently the library does not make use of any external dependencies - though hopefully
we will have BLAS/LAPACK bindings soon.

---

## Usage

The library usage is described well in the [API documentation](https://AtheMathmo.github.io/rulinalg/) - including example code.

### Installation

The library is most easily used with [cargo](http://doc.crates.io/guide.html). Simply include the following in your Cargo.toml file:

```toml
[dependencies]
rulinalg="0.4.2"
```

And then import the library using:

```rust
#[macro_use]
extern crate rulinalg;
```

Then import the modules and you're done!

```rust
use rulinalg::matrix::Matrix;

// Create a 2x2 matrix:
let a = Matrix::new(2, 2, vec![
    1.0, 2.0,
    3.0, 4.0,
]);

// Create a 2x3 matrix:
let b = Matrix::new(2, 3, vec![
    1.0, 2.0, 3.0,
    4.0, 5.0, 6.0,
]);

let c = &a * &b; // Matrix product of a and b

// Construct the product of `a` and `b` using the `matrix!` macro:
let expected = matrix![9.0, 12.0, 15.0;
                       19.0, 26.0, 33.0];

// Test for equality:
assert_matrix_eq!(c, expected);
```

More detailed coverage can be found in the [API documentation](https://AtheMathmo.github.io/rulinalg/).
