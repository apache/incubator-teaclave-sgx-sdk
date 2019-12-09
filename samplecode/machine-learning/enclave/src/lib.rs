// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

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
use std::time::*;
use std::untrusted::time::SystemTimeEx;

extern crate rusty_machine;
extern crate sgx_rand as rand;
extern crate serde;
extern crate serde_json;

use rusty_machine::linalg::{Matrix, BaseMatrix};
use rusty_machine::learning::k_means::{KMeansClassifier, KPlusPlus};
use rusty_machine::learning::UnSupModel;

use rand::thread_rng;
use rand::distributions::IndependentSample;
use rand::distributions::normal::Normal;
use rand::{random, Closed01};

use rusty_machine::learning::nnet::{NeuralNet, BCECriterion};
use rusty_machine::learning::toolkit::regularization::Regularization;
use rusty_machine::learning::toolkit::activ_fn::Sigmoid;
use rusty_machine::learning::optim::grad_desc::StochasticGD;

use rusty_machine::learning::SupModel;

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
    kmeans_sample();
    nn_sample();
    iris_sample();

    sgx_status_t::SGX_SUCCESS
}

fn iris_sample() {
    println!("IRIS classification sample:");
    // Set the layer sizes - from input to output
    let layers = &[4,10,10,1];

    println!("Layers (Input - Hiddeen - Output) {:?}", layers);
    const SAMPLES: usize = 10000000;
    let inputs: Vec<f64> = vec![5.1, 3.5, 1.4, 0.2,
                                    4.9, 3.0, 1.4, 0.2,
                                    4.7, 3.2, 1.3, 0.2,
                                    4.6, 3.1, 1.5, 0.2,
                                    5.0, 3.6, 1.4, 0.2,
                                    5.4, 3.9, 1.7, 0.4,
                                    4.6, 3.4, 1.4, 0.3,
                                    5.0, 3.4, 1.5, 0.2,
                                    4.4, 2.9, 1.4, 0.2,
                                    4.9, 3.1, 1.5, 0.1,
                                    5.4, 3.7, 1.5, 0.2,
                                    4.8, 3.4, 1.6, 0.2,
                                    4.8, 3.0, 1.4, 0.1,
                                    4.3, 3.0, 1.1, 0.1,
                                    5.8, 4.0, 1.2, 0.2,
                                    5.7, 4.4, 1.5, 0.4,
                                    5.4, 3.9, 1.3, 0.4,
                                    5.1, 3.5, 1.4, 0.3,
                                    5.7, 3.8, 1.7, 0.3,
                                    5.1, 3.8, 1.5, 0.3,
                                    5.4, 3.4, 1.7, 0.2,
                                    5.1, 3.7, 1.5, 0.4,
                                    4.6, 3.6, 1.0, 0.2,
                                    5.1, 3.3, 1.7, 0.5,
                                    4.8, 3.4, 1.9, 0.2,
                                    5.0, 3.0, 1.6, 0.2,
                                    5.0, 3.4, 1.6, 0.4,
                                    5.2, 3.5, 1.5, 0.2,
                                    5.2, 3.4, 1.4, 0.2,
                                    4.7, 3.2, 1.6, 0.2,
                                    4.8, 3.1, 1.6, 0.2,
                                    5.4, 3.4, 1.5, 0.4,
                                    5.2, 4.1, 1.5, 0.1,
                                    5.5, 4.2, 1.4, 0.2,
                                    4.9, 3.1, 1.5, 0.1,
                                    5.0, 3.2, 1.2, 0.2,
                                    5.5, 3.5, 1.3, 0.2,
                                    4.9, 3.1, 1.5, 0.1,
                                    4.4, 3.0, 1.3, 0.2,
                                    5.1, 3.4, 1.5, 0.2,
                                    5.0, 3.5, 1.3, 0.3,
                                    4.5, 2.3, 1.3, 0.3,
                                    4.4, 3.2, 1.3, 0.2,
                                    5.0, 3.5, 1.6, 0.6,
                                    5.1, 3.8, 1.9, 0.4,
                                    4.8, 3.0, 1.4, 0.3,
                                    5.1, 3.8, 1.6, 0.2,
                                    4.6, 3.2, 1.4, 0.2,
                                    5.3, 3.7, 1.5, 0.2,
                                    5.0, 3.3, 1.4, 0.2,
                                    7.0, 3.2, 4.7, 1.4,
                                    6.4, 3.2, 4.5, 1.5,
                                    6.9, 3.1, 4.9, 1.5,
                                    5.5, 2.3, 4.0, 1.3,
                                    6.5, 2.8, 4.6, 1.5,
                                    5.7, 2.8, 4.5, 1.3,
                                    6.3, 3.3, 4.7, 1.6,
                                    4.9, 2.4, 3.3, 1.0,
                                    6.6, 2.9, 4.6, 1.3,
                                    5.2, 2.7, 3.9, 1.4,
                                    5.0, 2.0, 3.5, 1.0,
                                    5.9, 3.0, 4.2, 1.5,
                                    6.0, 2.2, 4.0, 1.0,
                                    6.1, 2.9, 4.7, 1.4,
                                    5.6, 2.9, 3.6, 1.3,
                                    6.7, 3.1, 4.4, 1.4,
                                    5.6, 3.0, 4.5, 1.5,
                                    5.8, 2.7, 4.1, 1.0,
                                    6.2, 2.2, 4.5, 1.5,
                                    5.6, 2.5, 3.9, 1.1,
                                    5.9, 3.2, 4.8, 1.8,
                                    6.1, 2.8, 4.0, 1.3,
                                    6.3, 2.5, 4.9, 1.5,
                                    6.1, 2.8, 4.7, 1.2,
                                    6.4, 2.9, 4.3, 1.3,
                                    6.6, 3.0, 4.4, 1.4,
                                    6.8, 2.8, 4.8, 1.4,
                                    6.7, 3.0, 5.0, 1.7,
                                    6.0, 2.9, 4.5, 1.5,
                                    5.7, 2.6, 3.5, 1.0,
                                    5.5, 2.4, 3.8, 1.1,
                                    5.5, 2.4, 3.7, 1.0,
                                    5.8, 2.7, 3.9, 1.2,
                                    6.0, 2.7, 5.1, 1.6,
                                    5.4, 3.0, 4.5, 1.5,
                                    6.0, 3.4, 4.5, 1.6,
                                    6.7, 3.1, 4.7, 1.5,
                                    6.3, 2.3, 4.4, 1.3,
                                    5.6, 3.0, 4.1, 1.3,
                                    5.5, 2.5, 4.0, 1.3,
                                    5.5, 2.6, 4.4, 1.2,
                                    6.1, 3.0, 4.6, 1.4,
                                    5.8, 2.6, 4.0, 1.2,
                                    5.0, 2.3, 3.3, 1.0,
                                    5.6, 2.7, 4.2, 1.3,
                                    5.7, 3.0, 4.2, 1.2,
                                    5.7, 2.9, 4.2, 1.3,
                                    6.2, 2.9, 4.3, 1.3,
                                    5.1, 2.5, 3.0, 1.1,
                                    5.7, 2.8, 4.1, 1.3,
                                    6.3, 3.3, 6.0, 2.5,
                                    5.8, 2.7, 5.1, 1.9,
                                    7.1, 3.0, 5.9, 2.1,
                                    6.3, 2.9, 5.6, 1.8,
                                    6.5, 3.0, 5.8, 2.2,
                                    7.6, 3.0, 6.6, 2.1,
                                    4.9, 2.5, 4.5, 1.7,
                                    7.3, 2.9, 6.3, 1.8,
                                    6.7, 2.5, 5.8, 1.8,
                                    7.2, 3.6, 6.1, 2.5,
                                    6.5, 3.2, 5.1, 2.0,
                                    6.4, 2.7, 5.3, 1.9,
                                    6.8, 3.0, 5.5, 2.1,
                                    5.7, 2.5, 5.0, 2.0,
                                    5.8, 2.8, 5.1, 2.4,
                                    6.4, 3.2, 5.3, 2.3,
                                    6.5, 3.0, 5.5, 1.8,
                                    7.7, 3.8, 6.7, 2.2,
                                    7.7, 2.6, 6.9, 2.3,
                                    6.0, 2.2, 5.0, 1.5,
                                    6.9, 3.2, 5.7, 2.3,
                                    5.6, 2.8, 4.9, 2.0,
                                    7.7, 2.8, 6.7, 2.0,
                                    6.3, 2.7, 4.9, 1.8,
                                    6.7, 3.3, 5.7, 2.1,
                                    7.2, 3.2, 6.0, 1.8,
                                    6.2, 2.8, 4.8, 1.8,
                                    6.1, 3.0, 4.9, 1.8,
                                    6.4, 2.8, 5.6, 2.1,
                                    7.2, 3.0, 5.8, 1.6,
                                    7.4, 2.8, 6.1, 1.9,
                                    7.9, 3.8, 6.4, 2.0,
                                    6.4, 2.8, 5.6, 2.2,
                                    6.3, 2.8, 5.1, 1.5,
                                    6.1, 2.6, 5.6, 1.4,
                                    7.7, 3.0, 6.1, 2.3,
                                    6.3, 3.4, 5.6, 2.4,
                                    6.4, 3.1, 5.5, 1.8,
                                    6.0, 3.0, 4.8, 1.8,
                                    6.9, 3.1, 5.4, 2.1,
                                    6.7, 3.1, 5.6, 2.4,
                                    6.9, 3.1, 5.1, 2.3,
                                    5.8, 2.7, 5.1, 1.9,
                                    6.8, 3.2, 5.9, 2.3,
                                    6.7, 3.3, 5.7, 2.5,
                                    6.7, 3.0, 5.2, 2.3,
                                    6.3, 2.5, 5.0, 1.9,
                                    6.5, 3.0, 5.2, 2.0,
                                    6.2, 3.4, 5.4, 2.3,
                                    5.9, 3.0, 5.1, 1.8];

    let target: Vec<usize> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                  1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                                  1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                                  2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                                  2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];


    let target: Vec<f64> = target.into_iter().map(|x| x as f64).collect();

    let inputs = Matrix::new(150, 4, inputs);
    let targets = Matrix::new(150, 1, target);

    // Choose the BCE criterion with L2 regularization (`lambda=0.1`).
    let criterion = BCECriterion::new(Regularization::L2(0.1));

    // We will just use the default stochastic gradient descent.
    let mut model = NeuralNet::mlp(layers, criterion, StochasticGD::default(), Sigmoid);

    // Train the model!
    model.train(&inputs, &targets).unwrap();

    let test_cases = vec![ 5.9, 3.0, 5.1, 1.8];
    let test_inputs = Matrix::new(test_cases.len() / 4, 4, test_cases);

    println!("Infering {} times", SAMPLES);
    // start timer
    let now = SystemTime::now();
    for _ in 0..SAMPLES {
        // Predict
        let _ = model.predict(&test_inputs);
    }
    // end timer
    println!("Infer {} times: {:?}", SAMPLES, now.elapsed().unwrap());
}
fn nn_sample() {
    println!("AND gate learner sample:");

    const THRESHOLD: f64 = 0.7;

    const SAMPLES: usize = 10000;
    println!("Generating {} training data and labels...", SAMPLES as u32);

    let mut input_data = Vec::with_capacity(SAMPLES * 2);
    let mut label_data = Vec::with_capacity(SAMPLES);

    for _ in 0..SAMPLES {
        // The two inputs are "signals" between 0 and 1
        let Closed01(left) = random::<Closed01<f64>>();
        let Closed01(right) = random::<Closed01<f64>>();
        input_data.push(left);
        input_data.push(right);
        if left > THRESHOLD && right > THRESHOLD {
            label_data.push(1.0);
        } else {
            label_data.push(0.0)
        }
    }

    let inputs = Matrix::new(SAMPLES, 2, input_data);
    let targets = Matrix::new(SAMPLES, 1, label_data);

    let layers = &[2, 1];
    let criterion = BCECriterion::new(Regularization::L2(0.));
    // Create a multilayer perceptron with an input layer of size 2 and output layer of size 1
    // Uses a Sigmoid activation function and uses Stochastic gradient descent for training
    let mut model = NeuralNet::mlp(layers, criterion, StochasticGD::default(), Sigmoid);

    println!("Training...");
    // Our train function returns a Result<(), E>
    model.train(&inputs, &targets).unwrap();

    let test_cases = vec![
        0.0, 0.0,
        0.0, 1.0,
        1.0, 1.0,
        1.0, 0.0,
        ];
    let expected = vec![
        0.0,
        0.0,
        1.0,
        0.0,
        ];
    let test_inputs = Matrix::new(test_cases.len() / 2, 2, test_cases);
    let res = model.predict(&test_inputs).unwrap();

    println!("Evaluation...");
    let mut hits = 0;
    let mut misses = 0;
    // Evaluation
    println!("Got\tExpected");
    for (idx, prediction) in res.into_vec().iter().enumerate() {
        println!("{:.2}\t{}", prediction, expected[idx]);
        if (prediction - 0.5) * (expected[idx] - 0.5) > 0. {
            hits += 1;
        } else {
            misses += 1;
        }
    }

    println!("Hits: {}, Misses: {}", hits, misses);
    let hits_f = hits as f64;
    let total = (hits + misses) as f64;
    println!("Accuracy: {}%", (hits_f / total) * 100.);
}

fn kmeans_sample() {
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

    // Serialize the model to string
    let model_json = serde_json::to_string(&model).unwrap();
    println!("serialized model = {}", model_json);

    let centroids = model.centroids().as_ref().unwrap();
    println!("Model Centroids:\n{:.3}", centroids);

    // Predict the classes and partition into
    println!("Classifying the samples...");
    let classes = model.predict(&samples).unwrap();
    let (first, second): (Vec<usize>, Vec<usize>) = classes.data().iter().partition(|&x| *x == 0);

    println!("Samples closest to first centroid: {}", first.len());
    println!("Samples closest to second centroid: {}", second.len());

    let model_recovered : KMeansClassifier<KPlusPlus> = serde_json::from_str(&model_json).unwrap();
    println!("deserialized model = {:?}", model_recovered);

    let centroids = model_recovered.centroids().as_ref().unwrap();
    println!("Model Centroids:\n{:.3}", centroids);

    // Predict the classes and partition into
    println!("Classifying the samples using the deseralized model...");
    let classes = model_recovered.predict(&samples).unwrap();
    let (first, second): (Vec<usize>, Vec<usize>) = classes.data().iter().partition(|&x| *x == 0);

    println!("Samples closest to first centroid: {}", first.len());
    println!("Samples closest to second centroid: {}", second.len());
}
