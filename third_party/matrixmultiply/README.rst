matrixmultiply
==============

General matrix multiplication for f32, f64 matrices.

Allows arbitrary row, column strided matrices.

Uses the same microkernel algorithm as BLIS_, but in a much simpler
and less featureful implementation.
See their multithreading_ page for a very good diagram over how
the algorithm partitions the matrix (*Note:* this crate does not implement
multithreading).

.. _BLIS: https://github.com/flame/blis

.. _multithreading: https://github.com/flame/blis/wiki/Multithreading

Please read the `API documentation here`__

__ https://docs.rs/matrixmultiply/

Blog posts about this crate:

+ `A Gemmed Rabbit Hole`__

__ https://bluss.github.io/rust/2016/03/28/a-gemmed-rabbit-hole/

|build_status|_ |crates|_

.. |build_status| image:: https://travis-ci.org/bluss/matrixmultiply.svg?branch=master
.. _build_status: https://travis-ci.org/bluss/matrixmultiply

.. |crates| image:: https://meritbadge.herokuapp.com/matrixmultiply
.. _crates: https://crates.io/crates/matrixmultiply

**NOTE: Compile this crate using** ``RUSTFLAGS="-C target-cpu=native"`` **so
that the compiler can produce the best output.**

Recent Changes
--------------

- 0.1.14

  - Avoid an unused code warning

- 0.1.13

  - Pick 8x8 sgemm (f32) kernel when AVX target feature is enabled
    (with Rust 1.14 or later, no effect otherwise).
  - Use ``rawpointer``, a µcrate with raw pointer methods taken from this
    project.

- 0.1.12

  - Internal cleanup with retained performance

- 0.1.11

  - Adjust sgemm (f32) kernel to optimize better on recent Rust.

- 0.1.10

  - Update doc links to docs.rs

- 0.1.9

  - Workaround optimization regression in rust nightly (1.12-ish) (#9)

- 0.1.8

  - Improved docs

- 0.1.7

  - Reduce overhead slightly for small matrix multiplication problems by using
    only one allocation call for both packing buffers.

- 0.1.6

  - Disable manual loop unrolling in debug mode (quicker debug builds)

- 0.1.5

  - Update sgemm to use a 4x8 microkernel (“still in simplistic rust”),
    which improves throughput by 10%.

- 0.1.4

  - Prepare support for aligned packed buffers
  - Update dgemm to use a 8x4 microkernel, still in simplistic rust,
    which improves throughput by 10-20% when using AVX.

- 0.1.3

  - Silence some debug prints

- 0.1.2

  - Major performance improvement for sgemm and dgemm (20-30% when using AVX).
    Since it all depends on what the optimizer does, I'd love to get
    issue reports that report good or bad performance.
  - Made the kernel masking generic, which is a cleaner design

- 0.1.1

  - Minor improvement in the kernel
