// Copyright (c) 2016 rust-threshold-secret-sharing developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

#[macro_use]
extern crate bencher;
extern crate threshold_secret_sharing as tss;

mod shamir_vs_packed {

    use bencher::Bencher;
    use tss::shamir::*;

    pub fn bench_100_shamir(b: &mut Bencher) {
        let ref tss = ShamirSecretSharing {
            threshold: 155 / 3,
            parts: 728 / 3,
            prime: 746497,
        };

        let all_secrets: Vec<i64> = vec![5 ; 100 ];
        b.iter(|| {
            let _shares: Vec<Vec<i64>> = all_secrets.iter()
                .map(|&secret| tss.share(secret))
                .collect();
        });
    }

    pub fn bench_100_packed(b: &mut Bencher) {
        use tss::packed::*;
        let ref pss = PSS_155_728_100;
        let all_secrets: Vec<i64> = vec![5 ; 100];
        b.iter(|| {
            let _shares = pss.share(&all_secrets);
        })
    }

}

benchmark_group!(shamir_vs_packed,
                 shamir_vs_packed::bench_100_shamir,
                 shamir_vs_packed::bench_100_packed);


mod packed {

    use bencher::Bencher;
    use tss::packed::*;

    pub fn bench_large_secret_count(b: &mut Bencher) {
        let ref pss = PSS_155_728_100;
        let all_secrets = vec![5 ; pss.secret_count * 100];
        b.iter(|| {
            let _shares: Vec<Vec<i64>> = all_secrets.chunks(pss.secret_count)
                .map(|secrets| pss.share(&secrets))
                .collect();
        });
    }

    pub fn bench_large_share_count(b: &mut Bencher) {
        let ref pss = PSS_155_19682_100;
        let secrets = vec![5 ; pss.secret_count];
        b.iter(|| {
            let _shares = pss.share(&secrets);
        });
    }

    pub fn bench_large_reconstruct(b: &mut Bencher) {
        let ref pss = PSS_155_19682_100;
        let secrets = vec![5 ; pss.secret_count];
        let all_shares = pss.share(&secrets);

        // reconstruct using minimum number of shares required
        let indices: Vec<usize> = (0..pss.reconstruct_limit()).collect();
        let shares = &all_shares[0..pss.reconstruct_limit()];

        b.iter(|| {
            let _recovered_secrets = pss.reconstruct(&indices, &shares);
        });
    }

}

benchmark_group!(packed,
                 packed::bench_large_secret_count,
                 packed::bench_large_share_count,
                 packed::bench_large_reconstruct);

benchmark_main!(shamir_vs_packed, packed);
