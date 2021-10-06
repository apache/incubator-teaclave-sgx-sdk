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

#[link(name = "sgx_tstdc")]
extern "C" {
    pub fn acos(n: f64) -> f64;
    pub fn acosf(n: f32) -> f32;
    pub fn asin(n: f64) -> f64;
    pub fn asinf(n: f32) -> f32;
    pub fn atan(n: f64) -> f64;
    pub fn atan2(a: f64, b: f64) -> f64;
    pub fn atan2f(a: f32, b: f32) -> f32;
    pub fn atanf(n: f32) -> f32;
    pub fn cbrt(n: f64) -> f64;
    pub fn cbrtf(n: f32) -> f32;
    pub fn cosh(n: f64) -> f64;
    pub fn coshf(n: f32) -> f32;
    pub fn expm1(n: f64) -> f64;
    pub fn expm1f(n: f32) -> f32;
    pub fn fdim(a: f64, b: f64) -> f64;
    pub fn fdimf(a: f32, b: f32) -> f32;
    pub fn hypot(x: f64, y: f64) -> f64;
    pub fn hypotf(x: f32, y: f32) -> f32;
    pub fn log1p(n: f64) -> f64;
    pub fn log1pf(n: f32) -> f32;
    pub fn sinh(n: f64) -> f64;
    pub fn sinhf(n: f32) -> f32;
    pub fn tan(n: f64) -> f64;
    pub fn tanf(n: f32) -> f32;
    pub fn tanh(n: f64) -> f64;
    pub fn tanhf(n: f32) -> f32;
}