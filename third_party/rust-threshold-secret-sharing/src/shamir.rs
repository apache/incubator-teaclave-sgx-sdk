// Copyright (c) 2016 rust-threshold-secret-sharing developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Standard [Shamir secret sharing](https://en.wikipedia.org/wiki/Shamir%27s_Secret_Sharing)
//! for a single secret.

use rand;
use numtheory::*;
use std::vec::Vec;

/// Parameters for the Shamir scheme, specifying privacy threshold and total number of shares.
///
/// There are very few constraints except for the obvious ones:
///
/// * `prime` must be a prime large enough to hold the secrets we plan to share
/// * `share_count` must be at least `threshold + 1` (the reconstruction limit)
///
/// # Example:
///
/// ```
///    use threshold_secret_sharing::shamir;
///    let tss = shamir::ShamirSecretSharing {
///        threshold: 9,
///        share_count: 20,
///        prime: 41
///    };
///
///    let secret = 5;
///    let all_shares = tss.share(secret);
///
///    let reconstruct_share_count = tss.reconstruct_limit();
///
///    let indices: Vec<usize> = (0..reconstruct_share_count).collect();
///    let shares: &[i64] = &all_shares[0..reconstruct_share_count];
///    let recovered_secret = tss.reconstruct(&indices, shares);
///
///    println!("The recovered secret is {}", recovered_secret);
///    assert_eq!(recovered_secret, secret);
/// ```
#[derive(Debug)]
pub struct ShamirSecretSharing {
    /// Maximum number of shares that can be known without exposing the secret.
    pub threshold: usize,
    /// Number of shares to split the secret into.
    pub share_count: usize,
    /// Prime defining the Zp field in which computation is taking place.
    pub prime: i64,
}

/// Small preset parameters for tests.
pub static SHAMIR_5_20: ShamirSecretSharing = ShamirSecretSharing {
    threshold: 5,
    share_count: 20,
    prime: 41,
};

impl ShamirSecretSharing {
    /// Minimum number of shares required to reconstruct secret.
    ///
    /// For this scheme this is always `threshold + 1`.
    pub fn reconstruct_limit(&self) -> usize {
        self.threshold + 1
    }

    /// Generate `share_count` shares from `secret`.
    pub fn share(&self, secret: i64) -> Vec<i64> {
        let poly = self.sample_polynomial(secret);
        self.evaluate_polynomial(&poly)
    }

    /// Reconstruct `secret` from a large enough subset of the shares.
    ///
    /// `indices` are the ranks of the known shares as output by the `share` method,
    /// while `values` are the actual values of these shares.
    /// Both must have the same number of elements, and at least `reconstruct_limit`.
    pub fn reconstruct(&self, indices: &[usize], shares: &[i64]) -> i64 {
        assert!(shares.len() == indices.len());
        assert!(shares.len() >= self.reconstruct_limit());
        // add one to indices to get points
        let points: Vec<i64> = indices.iter().map(|&i| (i as i64) + 1i64).collect();
        lagrange_interpolation_at_zero(&*points, &shares, self.prime)
    }

    fn sample_polynomial(&self, zero_value: i64) -> Vec<i64> {
        // fix the first coefficient (corresponding to the evaluation at zero)
        let mut coefficients = vec![zero_value];
        // sample the remaining coefficients randomly using secure randomness
        use rand::distributions::Sample;
        let mut range = rand::distributions::range::Range::new(0, self.prime - 1);
        let mut rng = rand::SgxRng::new().unwrap();
        let random_coefficients: Vec<i64> =
            (0..self.threshold).map(|_| range.sample(&mut rng)).collect();
        coefficients.extend(random_coefficients);
        // return
        coefficients
    }

    fn evaluate_polynomial(&self, coefficients: &[i64]) -> Vec<i64> {
        // evaluate at all points
        (1..self.share_count + 1)
            .map(|point| mod_evaluate_polynomial(coefficients, point as i64, self.prime))
            .collect()
    }
}


#[test]
fn test_evaluate_polynomial() {
    let ref tss = SHAMIR_5_20;
    let poly = vec![1, 2, 0];
    let values = tss.evaluate_polynomial(&poly);
    assert_eq!(*values,
               [3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31, 33, 35, 37, 39, 0]);
}

#[test]
fn wikipedia_example() {
    let tss = ShamirSecretSharing {
        threshold: 2,
        share_count: 6,
        prime: 1613,
    };

    let shares = tss.evaluate_polynomial(&[1234, 166, 94]);
    assert_eq!(&*shares, &[1494, 329, 965, 176, 1188, 775]);

    assert_eq!(tss.reconstruct(&[0, 1, 2], &shares[0..3]), 1234);
    assert_eq!(tss.reconstruct(&[1, 2, 3], &shares[1..4]), 1234);
    assert_eq!(tss.reconstruct(&[2, 3, 4], &shares[2..5]), 1234);
}

#[test]
fn test_shamir() {
    let tss = ShamirSecretSharing {
        threshold: 2,
        share_count: 6,
        prime: 41,
    };
    let secret = 1;
    let shares = tss.share(secret);
    assert_eq!(tss.reconstruct(&[0, 1, 2], &shares[0..3]), secret);
    assert_eq!(tss.reconstruct(&[1, 2, 3], &shares[1..4]), secret);
    assert_eq!(tss.reconstruct(&[2, 3, 4, 5], &shares[2..6]), secret);
}
