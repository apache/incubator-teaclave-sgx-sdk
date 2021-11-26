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

pub fn test_fp64() {
    let f = 3.7_f64;
    let g = 3.0_f64;
    let h = -3.7_f64;
    
    assert_eq!(f.floor(), 3.0);
    assert_eq!(g.floor(), 3.0);
    assert_eq!(h.floor(), -4.0);

    let f = 3.01_f64;
    let g = 4.0_f64;
    assert_eq!(f.ceil(), 4.0);
    assert_eq!(g.ceil(), 4.0);
    
    let f = 3.3_f64;
    let g = -3.3_f64;
    assert_eq!(f.round(), 3.0);
    assert_eq!(g.round(), -3.0);
    
    let f = 3.7_f64;
    let g = 3.0_f64;
    let h = -3.7_f64;
    assert_eq!(f.trunc(), 3.0);
    assert_eq!(g.trunc(), 3.0);
    assert_eq!(h.trunc(), -3.0);
    
    let x = 3.6_f64;
    let y = -3.6_f64;
    let abs_difference_x = (x.fract() - 0.6).abs();
    let abs_difference_y = (y.fract() - (-0.6)).abs();
    assert!(abs_difference_x < 1e-10);
    assert!(abs_difference_y < 1e-10);
    
    let x = 3.5_f64;
    let y = -3.5_f64;
    let abs_difference_x = (x.abs() - x).abs();
    let abs_difference_y = (y.abs() - (-y)).abs();
    assert!(abs_difference_x < 1e-10);
    assert!(abs_difference_y < 1e-10);
    assert!(f64::NAN.abs().is_nan());

    let f = 3.5_f64;
    assert_eq!(f.signum(), 1.0);
    assert_eq!(f64::NEG_INFINITY.signum(), -1.0);
    assert!(f64::NAN.signum().is_nan());
    
    let f = 3.5_f64;
    assert_eq!(f.copysign(0.42), 3.5_f64);
    assert_eq!(f.copysign(-0.42), -3.5_f64);
    assert_eq!((-f).copysign(0.42), 3.5_f64);
    assert_eq!((-f).copysign(-0.42), -3.5_f64);
    assert!(f64::NAN.copysign(1.0).is_nan());
    
    let m = 10.0_f64;
    let x = 4.0_f64;
    let b = 60.0_f64;
    // 100.0
    let abs_difference = (m.mul_add(x, b) - ((m * x) + b)).abs();
    assert!(abs_difference < 1e-10);
    
    let a: f64 = 7.0;
    let b = 4.0;
    assert_eq!(a.div_euclid(b), 1.0); // 7.0 > 4.0 * 1.0
    assert_eq!((-a).div_euclid(b), -2.0); // -7.0 >= 4.0 * -2.0
    assert_eq!(a.div_euclid(-b), -1.0); // 7.0 >= -4.0 * -1.0
    assert_eq!((-a).div_euclid(-b), 2.0); // -7.0 >= -4.0 * 2.0
    
    let a: f64 = 7.0;
    let b = 4.0;
    assert_eq!(a.rem_euclid(b), 3.0);
    assert_eq!((-a).rem_euclid(b), 1.0);
    assert_eq!(a.rem_euclid(-b), 3.0);
    assert_eq!((-a).rem_euclid(-b), 1.0);
    // limitation due to round-off error
    assert!((-f64::EPSILON).rem_euclid(3.0) != 0.0);
    
    let x = 2.0_f64;
    let abs_difference = (x.powi(2) - (x * x)).abs();
    assert!(abs_difference < 1e-10);
    
    let x = 2.0_f64;
    let abs_difference = (x.powf(2.0) - (x * x)).abs();
    assert!(abs_difference < 1e-10);

    let positive = 4.0_f64;
    let negative = -4.0_f64;
    let abs_difference = (positive.sqrt() - 2.0).abs();
    assert!(abs_difference < 1e-10);
    assert!(negative.sqrt().is_nan());
    
    let one = 1.0_f64;
    // e^1
    let e = one.exp();
    // ln(e) - 1 == 0
    let abs_difference = (e.ln() - 1.0).abs();
    assert!(abs_difference < 1e-10);
    
    let f = 2.0_f64;
    // 2^2 - 4 == 0
    let abs_difference = (f.exp2() - 4.0).abs();
    assert!(abs_difference < 1e-10);
    
    let one = 1.0_f64;
    // e^1
    let e = one.exp();
    // ln(e) - 1 == 0
    let abs_difference = (e.ln() - 1.0).abs();
    assert!(abs_difference < 1e-10);
    
    let twenty_five = 25.0_f64;
    // log5(25) - 2 == 0
    let abs_difference = (twenty_five.log(5.0) - 2.0).abs();
    assert!(abs_difference < 1e-10);
    
    let four = 4.0_f64;
    // log2(4) - 2 == 0
    let abs_difference = (four.log2() - 2.0).abs();
    assert!(abs_difference < 1e-10);
    
    let hundred = 100.0_f64;
    // log10(100) - 2 == 0
    let abs_difference = (hundred.log10() - 2.0).abs();
    assert!(abs_difference < 1e-10);
    
    let x = 3.0_f64;
    let y = -3.0_f64;
    let abs_difference_x = (x.abs_sub(1.0) - 2.0).abs();
    let abs_difference_y = (y.abs_sub(1.0) - 0.0).abs();
    assert!(abs_difference_x < 1e-10);
    assert!(abs_difference_y < 1e-10);
    
    let x = 8.0_f64;
    // x^(1/3) - 2 == 0
    let abs_difference = (x.cbrt() - 2.0).abs();
    assert!(abs_difference < 1e-10);
    
    let x = 2.0_f64;
    let y = 3.0_f64;
    // sqrt(x^2 + y^2)
    let abs_difference = (x.hypot(y) - (x.powi(2) + y.powi(2)).sqrt()).abs();
    assert!(abs_difference < 1e-10);
    
    let x = std::f64::consts::FRAC_PI_2;
    let abs_difference = (x.sin() - 1.0).abs();
    assert!(abs_difference < 1e-10);
    
    let x = 2.0 * std::f64::consts::PI;
    let abs_difference = (x.cos() - 1.0).abs();
    assert!(abs_difference < 1e-10);
    
    let f = std::f64::consts::FRAC_PI_2;
    // asin(sin(pi/2))
    let abs_difference = (f.sin().asin() - std::f64::consts::FRAC_PI_2).abs();
    assert!(abs_difference < 1e-10);
    
    let f = std::f64::consts::FRAC_PI_4;
    // acos(cos(pi/4))
    let abs_difference = (f.cos().acos() - std::f64::consts::FRAC_PI_4).abs();
    assert!(abs_difference < 1e-10);
    
    let f = 1.0_f64;
    // atan(tan(1))
    let abs_difference = (f.tan().atan() - 1.0).abs();
    assert!(abs_difference < 1e-10);

    // Positive angles measured counter-clockwise
    // from positive x axis
    // -pi/4 radians (45 deg clockwise)
    let x1 = 3.0_f64;
    let y1 = -3.0_f64;
    // 3pi/4 radians (135 deg counter-clockwise)
    let x2 = -3.0_f64;
    let y2 = 3.0_f64;
    let abs_difference_1 = (y1.atan2(x1) - (-std::f64::consts::FRAC_PI_4)).abs();
    let abs_difference_2 = (y2.atan2(x2) - (3.0 * std::f64::consts::FRAC_PI_4)).abs();
    assert!(abs_difference_1 < 1e-10);
    assert!(abs_difference_2 < 1e-10);
    
    let x = std::f64::consts::FRAC_PI_4;
    let f = x.sin_cos();
    let abs_difference_0 = (f.0 - x.sin()).abs();
    let abs_difference_1 = (f.1 - x.cos()).abs();
    assert!(abs_difference_0 < 1e-10);
    assert!(abs_difference_1 < 1e-10);
    
    let x = 1e-16_f64;
    // for very small x, e^x is approximately 1 + x + x^2 / 2
    let approx = x + x * x / 2.0;
    let abs_difference = (x.exp_m1() - approx).abs();
    assert!(abs_difference < 1e-20);
    
    let x = 1e-16_f64;
    // for very small x, ln(1 + x) is approximately x - x^2 / 2
    let approx = x - x * x / 2.0;
    let abs_difference = (x.ln_1p() - approx).abs();
    assert!(abs_difference < 1e-20);
    
    let e = std::f64::consts::E;
    let x = 1.0_f64;
    let f = x.sinh();
    // Solving sinh() at 1 gives `(e^2-1)/(2e)`
    let g = ((e * e) - 1.0) / (2.0 * e);
    let abs_difference = (f - g).abs();
    assert!(abs_difference < 1e-10);
    
    let e = std::f64::consts::E;
    let x = 1.0_f64;
    let f = x.cosh();
    // Solving cosh() at 1 gives this result
    let g = ((e * e) + 1.0) / (2.0 * e);
    let abs_difference = (f - g).abs();
    // Same result
    assert!(abs_difference < 1.0e-10);
    
    let e = std::f64::consts::E;
    let x = 1.0_f64;
    let f = x.tanh();
    // Solving tanh() at 1 gives `(1 - e^(-2))/(1 + e^(-2))`
    let g = (1.0 - e.powi(-2)) / (1.0 + e.powi(-2));
    let abs_difference = (f - g).abs();
    assert!(abs_difference < 1.0e-10);
    
    let x = 1.0_f64;
    let f = x.sinh().asinh();
    let abs_difference = (f - x).abs();
    assert!(abs_difference < 1.0e-10);
    
    let x = 1.0_f64;
    let f = x.cosh().acosh();
    let abs_difference = (f - x).abs();
    assert!(abs_difference < 1.0e-10);
    
    let e = std::f64::consts::E;
    let f = e.tanh().atanh();
    let abs_difference = (f - e).abs();
    assert!(abs_difference < 1.0e-10);
}
