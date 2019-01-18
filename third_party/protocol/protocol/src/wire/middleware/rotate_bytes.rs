//! A fixed-offset based caesar cipher middleware.

use Error;
use wire;

use std::prelude::v1::*;
use std::num::Wrapping;

/// Middleware that rotates each transmitted byte by a fixed offset.
///
/// **NOTE**: This is not really useful in real life.
#[derive(Copy, Clone, Debug)]
pub struct RotateBytes {
    /// The integer offset to rotate by.
    pub amount: u8,
}

impl RotateBytes {
    /// The ROT13 cipher.
    ///
    /// [ROT13 on Wikipedia](https://en.wikipedia.org/wiki/ROT13)
    pub const ROT13: RotateBytes = RotateBytes { amount: 13 };
}

impl wire::Middleware for RotateBytes {
    fn encode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error> {
        Ok(data.into_iter().map(|b| (Wrapping(b) + Wrapping(self.amount)).0).collect())
    }

    fn decode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error> {
        Ok(data.into_iter().map(|b| (Wrapping(b) - Wrapping(self.amount)).0).collect())
    }
}

#[cfg(test)]
mod test {
    use super::RotateBytes;
    use wire::Middleware;

    #[test]
    fn bytes_encoded_correctly() {
        assert_eq!(RotateBytes::ROT13.encode_data(vec![0, 1, 2, 3, 4]).unwrap(),
                   vec![13, 14, 15, 16, 17]);
    }

    #[test]
    fn bytes_decoded_correctly() {
        assert_eq!(RotateBytes::ROT13.decode_data(vec![13, 14, 15, 16, 17]).unwrap(),
                   vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn bytes_read_back_correctly() {
        assert_eq!(RotateBytes::ROT13.encode_data(RotateBytes::ROT13.decode_data(vec![2, 4, 8, 32, 254]).unwrap()).unwrap(),
                   vec![2, 4, 8, 32, 254]);
    }
}

