// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![crate_name = "machinelearningsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use sgx_types::*;
use std::vec::Vec;

extern crate rusty_machine;
extern crate sgx_rand as rand;

use rusty_machine::linalg::{Matrix, BaseMatrix};
use rusty_machine::learning::k_means::KMeansClassifier;
use rusty_machine::learning::UnSupModel;

use rand::thread_rng;
use rand::distributions::IndependentSample;
use rand::distributions::normal::Normal;

fn generate_data(centroids: &Matrix<f64>,
                 points_per_centroid: usize,
                 noise: f64)
                 -> Matrix<f64> {
    assert!(centroids.cols() > 0, "Centroids cannot be empty.");
    assert!(centroids.rows() > 0, "Centroids cannot be empty.");
    assert!(noise >= 0f64, "Noise must be non-negative.");
    let mut raw_cluster_data = Vec::with_capacity(centroids.rows() * points_per_centroid *
                                                  centroids.cols());

    let mut rng = thread_rng();
    let normal_rv = Normal::new(0f64, noise);

    for _ in 0..points_per_centroid {
        // Generate points from each centroid
        for centroid in centroids.row_iter() {
            // Generate a point randomly around the centroid
            let mut point = Vec::with_capacity(centroids.cols());
            for feature in centroid.iter() {
                point.push(feature + normal_rv.ind_sample(&mut rng));
            }

            // Push point to raw_cluster_data
            raw_cluster_data.extend(point);
        }
    }

    Matrix::new(centroids.rows() * points_per_centroid,
                centroids.cols(),
                raw_cluster_data)
}

#[no_mangle]
pub extern "C"
fn sample_main() -> sgx_status_t {
    println!("K-Means clustering example:");

    const SAMPLES_PER_CENTROID: usize = 2000;

    println!("Generating {0} samples from each centroids:",
             SAMPLES_PER_CENTROID);
    // Choose two cluster centers, at (-0.5, -0.5) and (0, 0.5).
    let centroids = Matrix::new(2, 2, vec![-0.5, -0.5, 0.0, 0.5]);
    println!("{}", centroids);

    // Generate some data randomly around the centroids
    let samples = generate_data(&centroids, SAMPLES_PER_CENTROID, 0.4);

    // Create a new model with 2 clusters
    let mut model = KMeansClassifier::new(2);

    // Train the model
    println!("Training the model...");
    // Our train function returns a Result<(), E>
    model.train(&samples).unwrap();

    let centroids = model.centroids().as_ref().unwrap();
    println!("Model Centroids:\n{:.3}", centroids);

    // Predict the classes and partition into
    println!("Classifying the samples...");
    let classes = model.predict(&samples).unwrap();
    let (first, second): (Vec<usize>, Vec<usize>) = classes.data().iter().partition(|&x| *x == 0);

    println!("Samples closest to first centroid: {}", first.len());
    println!("Samples closest to second centroid: {}", second.len());

    sgx_status_t::SGX_SUCCESS
}
