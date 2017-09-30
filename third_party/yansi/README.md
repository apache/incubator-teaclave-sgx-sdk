# yansi

[![Current Crates.io Version](https://img.shields.io/crates/v/yansi.svg)](https://crates.io/crates/yansi)
[![Documentation](https://docs.rs/yansi/badge.svg)](https://docs.rs/yansi)

A dead simple ANSI terminal color painting library for Rust.

```rust
use yansi::Paint;

print!("{} light, {} light!", Paint::green("Green"), Paint::red("red").underline());
```

See the [documentation](https://docs.rs/yansi) for more.

## Why?

Several terminal coloring libraries already exist for Rust ([`ansi_term`],
[`colored`], [`term_painter`], to name a few), begging the question: why yet
another? Here are a few reasons for `yansi`:

  * This library is _much_ simpler: there are two types! The complete
    implementation is only about 250 lines of code.
  * Like [`term_painter`], but unlike [`ansi_term`], _any_ type implementing
    `Display` can be stylized, not only strings.
  * Styling can be enabled and disabled on the fly.
  * Arbitrary items can be _masked_ for selective disabling.
  * Typically, only one type needs to be imported: `Paint`.
  * Zero dependencies. It really is simple.
  * The name `yansi` is pretty short.

All that being said, this library borrows the general API of existing libraries
as well as plenty of code from [`ansi_term`].

[`ansi_term`]: https://crates.io/crates/ansi_term
[`colored`]: https://crates.io/crates/colored
[`term_painter`]: https://crates.io/crates/term-painter

## License

`yansi` is licensed under either of the following, at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
