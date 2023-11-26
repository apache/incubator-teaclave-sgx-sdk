//! This module contains the implementation of the `eh_personality` lang item.
//!
//! The actual implementation is heavily dependent on the target since Rust
//! tries to use the native stack unwinding mechanism whenever possible.
//!
//! This personality function is still required with `-C panic=abort` because
//! it is used to catch foreign exceptions from `extern "C-unwind"` and turn
//! them into aborts.
//!
//! Additionally, ARM EHABI uses the personality function when generating
//! backtraces.

mod dwarf;
mod gcc;
