//! Stream-based and datagram-based communication implementations.

pub use self::middleware::Middleware;

/// Stream-based over the wire communication.
pub mod stream;
/// Datagram-based over the wire communication.
pub mod dgram;

#[macro_use]
pub mod middleware;

