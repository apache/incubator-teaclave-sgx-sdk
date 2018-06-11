// Copyright (c) 2016 rust-threshold-secret-sharing developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.
extern crate threshold_secret_sharing as tss;

fn main() {

    let ref pss = tss::packed::PSS_4_26_3;
    println!("\
    Using parameters that: \n \
     - allow {} values to be packed together \n \
     - give a security threshold of {} \n \
     - require {} of the {} shares to reconstruct in the basic case",
        pss.secret_count,
        pss.threshold,
        pss.reconstruct_limit(),
        pss.share_count
    );

    // define inputs
    let secrets_1 = vec![1, 2, 3];
    println!("\nFirst input vector:  {:?}", &secrets_1);
    let secrets_2 = vec![4, 5, 6];
    println!("Second input vector: {:?}", &secrets_2);
    let secrets_3 = vec![3, 2, 1];
    println!("Third input vector:  {:?}", &secrets_3);
    let secrets_4 = vec![6, 5, 4];
    println!("Fourth input vector: {:?}", &secrets_4);

    // secret share inputs
    let shares_1 = pss.share(&secrets_1);
    println!("\nSharing of first vector gives random shares S1:\n{:?}", &shares_1);
    let shares_2 = pss.share(&secrets_2);
    println!("\nSharing of second vector gives random shares S2:\n{:?}", &shares_2);
    let shares_3 = pss.share(&secrets_3);
    println!("\nSharing of third vector gives random shares S3:\n{:?}", &shares_3);
    let shares_4 = pss.share(&secrets_4);
    println!("\nSharing of fourth vector gives random shares S4:\n{:?}", &shares_4);

    // in the following, 'positivise' is used to map (potentially negative)
    // values to their equivalent positive representation in Z_p for usability
    use tss::positivise;

    // multiply shares_1 and shares_2 point-wise
    let shares_12: Vec<i64> = shares_1.iter().zip(&shares_2).map(|(a, b)| (a * b) % pss.prime).collect();
    // ... and reconstruct product, using double reconstruction limit
    let shares_12_reconstruct_limit = pss.reconstruct_limit() * 2;
    let foo: Vec<usize> = (0..shares_12_reconstruct_limit).collect();
    let bar = &shares_12[0..shares_12_reconstruct_limit];
    let secrets_12 = pss.reconstruct(&foo, bar);
    println!(
        "\nMultiplying shares S1 and S2 point-wise gives new shares S12 which \
        can be reconstructed (using {} of them) to give output vector: {:?}",
        shares_12_reconstruct_limit,
        positivise(&secrets_12, pss.prime)
    );

    // multiply shares_3 and shares_4 point-wise
    let shares_34: Vec<i64> = shares_3.iter().zip(&shares_4).map(|(a, b)| (a * b) % pss.prime).collect();
    // ... and reconstruct product, using double reconstruction limit
    let shares_34_reconstruct_limit = pss.reconstruct_limit() * 2;
    let foo: Vec<usize> = (0..shares_34_reconstruct_limit).collect();
    let bar = &shares_34[0..shares_34_reconstruct_limit];
    let secrets_34 = pss.reconstruct(&foo, bar);
    println!(
        "\nLikewise, multiplying shares S3 and S4 point-wise gives new shares S34 \
        which can be reconstructed (using {} of them) to give output vector: {:?}",
        shares_34_reconstruct_limit,
        positivise(&secrets_34, pss.prime)
    );

    // multiply shares_sum12 and shares_34 point-wise
    let shares_1234product: Vec<i64> = shares_12.iter().zip(&shares_34).map(|(a, b)| (a * b) % pss.prime).collect();
    // ... and reconstruct product, using double reconstruction limit
    let shares_1234product_reconstruct_limit = shares_1234product.len();
    let foo: Vec<usize> = (0..shares_1234product_reconstruct_limit).collect();
    let bar = &shares_1234product[0..shares_1234product_reconstruct_limit];
    let secrets_1234product = pss.reconstruct(&foo, bar);
    println!(
        "\nIf we continue multiplying these new shares S12 and S34 then we no longer \
        have enough shares to reconstruct correctly; using all {} shares gives incorrect (random) \
        output: {:?}",
        shares_1234product_reconstruct_limit,
        positivise(&secrets_1234product, pss.prime)
    );

    // add shares_12 and shares_34 point-wise
    let shares_1234sum: Vec<i64> = shares_12.iter().zip(&shares_34).map(|(a, b)| (a + b) % pss.prime).collect();
    // ... and reconstruct sum, using same reconstruction limit as inputs
    let shares_1234sum_reconstruct_limit = pss.reconstruct_limit() * 2;
    let foo: Vec<usize> = (0..shares_1234sum_reconstruct_limit).collect();
    let bar = &shares_1234sum[0..shares_1234sum_reconstruct_limit];
    let secrets_1234sum = pss.reconstruct(&foo, bar);
    println!(
        "\nHowever, adding shares S12 and S34 point-wise doesn't increase the \
        reconstruction limit and hence using {} shares we can still recover their sum: {:?}",
        shares_1234sum_reconstruct_limit,
        positivise(&secrets_1234sum, pss.prime)
    );

}
