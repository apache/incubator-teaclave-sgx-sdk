// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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

#![crate_name = "machinelearningenclave"]
#![crate_type = "staticlib"]

#![no_std]

extern crate sgx_types;
#[macro_use]
extern crate sgx_tstd as std;

use sgx_types::*;
use std::vec::Vec;

extern crate rusty_machine;
extern crate sgx_rand as rand;

use rusty_machine::linalg::{Matrix, BaseMatrix};
use rusty_machine::learning::k_means::KMeansClassifier;
use rusty_machine::learning::UnSupModel;

use rusty_machine::learning::naive_bayes::{self, NaiveBayes};
use rusty_machine::learning::SupModel;

use rand::thread_rng;
use rand::distributions::IndependentSample;
use rand::distributions::normal::Normal;
use rand::Rand;
use rand::distributions::Sample;

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
fn k_means_main() -> sgx_status_t {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Color {
    Red,
    White,
}

#[derive(Clone, Debug)]
struct Dog {
    color: Color,
    friendliness: f64,
    furriness: f64,
    speed: f64,
}

impl Rand for Dog {
    /// Generate a random dog.
    fn rand<R: rand::Rng>(rng: &mut R) -> Self {
        // Friendliness, furriness, and speed are normally distributed and
        // (given color:) independent.
        let mut red_dog_friendliness = Normal::new(0., 1.);
        let mut red_dog_furriness = Normal::new(0., 1.);
        let mut red_dog_speed = Normal::new(0., 1.);

        let mut white_dog_friendliness = Normal::new(1., 1.);
        let mut white_dog_furriness = Normal::new(1., 1.);
        let mut white_dog_speed = Normal::new(-1., 1.);

        // Flip a coin to decide whether to generate a red or white dog.
        let coin: f64 = rng.gen();
        let color = if coin < 0.5 { Color::Red } else { Color::White };

        match color {
            Color::Red => {
                Dog {
                    color: Color::Red,
                    // sample from our normal distributions for each trait
                    friendliness: red_dog_friendliness.sample(rng),
                    furriness: red_dog_furriness.sample(rng),
                    speed: red_dog_speed.sample(rng),
                }
            },
            Color::White => {
                Dog {
                    color: Color::White,
                    friendliness: white_dog_friendliness.sample(rng),
                    furriness: white_dog_furriness.sample(rng),
                    speed: white_dog_speed.sample(rng),
                }
            },
        }
    }
}

fn generate_dog_data(training_set_size: u32, test_set_size: u32)
    -> (Matrix<f64>, Matrix<f64>, Matrix<f64>, Vec<Dog>) {
    let mut randomness = rand::StdRng::new()
        .expect("we should be able to get an RNG");
    let rng = &mut randomness;

    // We'll train the model on these dogs
    let training_dogs = (0..training_set_size)
        .map(|_| { Dog::rand(rng) })
        .collect::<Vec<_>>();

    // ... and then use the model to make predictions about these dogs' color
    // given only their trait measurements.
    let test_dogs = (0..test_set_size)
        .map(|_| { Dog::rand(rng) })
        .collect::<Vec<_>>();

    // The model's `.train` method will take two matrices, each with a row for
    // each dog in the training set: the rows in the first matrix contain the
    // trait measurements; the rows in the second are either [1, 0] or [0, 1]
    // to indicate color.
    let training_data: Vec<f64> = training_dogs.iter()
        .flat_map(|dog| vec![dog.friendliness, dog.furriness, dog.speed])
        .collect();
    let training_matrix: Matrix<f64> = training_data.chunks(3).collect();
    let target_data: Vec<f64> = training_dogs.iter()
        .flat_map(|dog| match dog.color {
            Color::Red => vec![1., 0.],
            Color::White => vec![0., 1.],
        })
        .collect();
    let target_matrix: Matrix<f64> = target_data.chunks(2).collect();

    // Build another matrix for the test set of dogs to make predictions about.
    let test_data: Vec<f64> = test_dogs.iter()
        .flat_map(|dog| vec![dog.friendliness, dog.furriness, dog.speed])
        .collect();
    let test_matrix: Matrix<f64> = test_data.chunks(3).collect();

    (training_matrix, target_matrix, test_matrix, test_dogs)
}

fn evaluate_prediction(hits: &mut u32, dog: &Dog, prediction: &[f64]) -> (Color, bool) {
    let predicted_color = dog.color;
    let actual_color = if prediction[0] == 1. {
        Color::Red
    } else {
        Color::White
    };
    let accurate = predicted_color == actual_color;
    if accurate {
        *hits += 1;
    }
    (actual_color, accurate)
}

#[no_mangle]
pub extern "C"
fn naive_bayes_dogs_main() {
    let (training_set_size, test_set_size) = (1000, 1000);
    // Generate all of our train and test data
    let (training_matrix, target_matrix, test_matrix, test_dogs) = generate_dog_data(training_set_size, test_set_size);

    // Train!
    let mut model = NaiveBayes::<naive_bayes::Gaussian>::new();
    model.train(&training_matrix, &target_matrix)
        .expect("failed to train model of dogs");

    // Predict!
    let predictions = model.predict(&test_matrix)
        .expect("failed to predict dogs!?");

    // Score how well we did.
    let mut hits = 0;
    let unprinted_total = test_set_size.saturating_sub(10) as usize;
    for (dog, prediction) in test_dogs.iter().zip(predictions.row_iter()).take(unprinted_total) {
        evaluate_prediction(&mut hits, dog, prediction.raw_slice());
    }

    if unprinted_total > 0 {
        println!("...");
    }

    for (dog, prediction) in test_dogs.iter().zip(predictions.row_iter()).skip(unprinted_total) {
        let (actual_color, accurate) = evaluate_prediction(&mut hits, dog, prediction.raw_slice());
        println!("Predicted: {:?}; Actual: {:?}; Accurate? {:?}",
                 dog.color, actual_color, accurate);
    }

    println!("Accuracy: {}/{} = {:.1}%", hits, test_set_size,
             (f64::from(hits))/(f64::from(test_set_size)) * 100.);
}

use rand::{random,Closed01};

use rusty_machine::learning::nnet::{NeuralNet, BCECriterion};
use rusty_machine::learning::toolkit::regularization::Regularization;
use rusty_machine::learning::toolkit::activ_fn::Sigmoid;
use rusty_machine::learning::optim::grad_desc::StochasticGD;

// AND gate
#[no_mangle]
pub extern "C"
fn and_gate_main() {
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

use rusty_machine::learning::svm::SVM;
// Necessary for the training trait.
use rusty_machine::learning::toolkit::kernel::HyperTan;

use rusty_machine::linalg::Vector;

// Sign learner:
//   * Model input a float number
//   * Model output: A float representing the input sign.
//       If the input is positive, the output is close to 1.0.
//       If the input is negative, the output is close to -1.0.
//   * Model generated with the SVM API.
#[no_mangle]
pub extern "C"
fn svm_sign_learner_main() {
    println!("Sign learner sample:");

    println!("Training...");
    // Training data
    let inputs = Matrix::new(11, 1, vec![
                             -0.1, -2., -9., -101., -666.7,
                             0., 0.1, 1., 11., 99., 456.7
                             ]);
    let targets = Vector::new(vec![
                              -1., -1., -1., -1., -1.,
                              1., 1., 1., 1., 1., 1.
                              ]);

    // Trainee
    let mut svm_mod = SVM::new(HyperTan::new(100., 0.), 0.3);
    // Our train function returns a Result<(), E>
    svm_mod.train(&inputs, &targets).unwrap();

    println!("Evaluation...");
    let mut hits = 0;
    let mut misses = 0;
    // Evaluation
    //   Note: We could pass all input values at once to the `predict` method!
    //         Here, we use a loop just to count and print logs.
    for n in (-1000..1000).filter(|&x| x % 100 == 0) {
        let nf = n as f64;
        let input = Matrix::new(1, 1, vec![nf]);
        let out = svm_mod.predict(&input).unwrap();
        let res = if out[0] * nf > 0. {
            hits += 1;
            true
        } else if nf == 0. {
            hits += 1;
            true
        } else {
            misses += 1;
            false
        };

        println!("{} -> {}: {}", Matrix::data(&input)[0], out[0], res);
    }

    println!("Performance report:");
    println!("Hits: {}, Misses: {}", hits, misses);
    let hits_f = hits as f64;
    let total = (hits + misses) as f64;
    println!("Accuracy: {}", (hits_f / total) * 100.);
}
