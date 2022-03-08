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

use crate::stats;
use crate::time::Instant;
use alloc::string::String;
use core::cmp;
use core::time::Duration;

/// An identity function that *__hints__* to the compiler to be maximally pessimistic about what
/// `black_box` could do.
///
/// See [`std::hint::black_box`] for details.
#[inline(always)]
pub fn black_box<T>(dummy: T) -> T {
    core::hint::black_box(dummy)
}

/// Manager of the benchmarking runs.
///
/// This is fed into functions marked with `#[bench]` to allow for
/// set-up & tear-down before running a piece of code repeatedly via a
/// call to `iter`.
#[derive(Clone)]
pub struct Bencher {
    summary: Option<stats::Summary>,
    freq: u64,
    pub bytes: u64,
}

impl Bencher {
    pub fn new(freq: u64) -> Bencher {
        Bencher {
            summary: None,
            bytes: 0,
            freq,
        }
    }

    /// Callback for benchmark functions to run in their body.
    pub fn iter<T, F>(&mut self, mut inner: F)
    where
        F: FnMut() -> T,
    {
        self.summary = Some(iter(self.freq, &mut inner));
    }

    pub fn bench<F>(&mut self, mut f: F) -> Option<stats::Summary>
    where
        F: FnMut(&mut Bencher),
    {
        f(self);
        self.summary
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchSamples {
    pub ns_iter_summ: stats::Summary,
    pub mb_s: usize,
}

pub fn fmt_bench_samples(bs: &BenchSamples) -> String {
    use core::fmt::Write;
    let mut output = String::new();

    let median = bs.ns_iter_summ.median as usize;
    let deviation = (bs.ns_iter_summ.max - bs.ns_iter_summ.min) as usize;

    write!(
        output,
        "{:>11} ns/iter (+/- {})",
        fmt_thousands_sep(median, ','),
        fmt_thousands_sep(deviation, ',')
    )
    .unwrap();
    if bs.mb_s != 0 {
        write!(output, " = {} MB/s", bs.mb_s).unwrap();
    }
    output
}

// Format a number with thousands separators
fn fmt_thousands_sep(mut n: usize, sep: char) -> String {
    use core::fmt::Write;
    let mut output = String::new();
    let mut trailing = false;
    for &pow in &[9, 6, 3, 0] {
        let base = 10_usize.pow(pow);
        if pow == 0 || trailing || n / base != 0 {
            if !trailing {
                write!(output, "{}", n / base).unwrap();
            } else {
                write!(output, "{:03}", n / base).unwrap();
            }
            if pow != 0 {
                output.push(sep);
            }
            trailing = true;
        }
        n %= base;
    }

    output
}

fn ns_iter_inner<T, F>(freq: u64, inner: &mut F, k: u64) -> u64
where
    F: FnMut() -> T,
{
    let start = Instant::now(freq);
    for _ in 0..k {
        black_box(inner());
    }
    start.elapsed().as_nanos() as u64
}

fn iter<T, F>(freq: u64, inner: &mut F) -> stats::Summary
where
    F: FnMut() -> T,
{
    // Initial bench run to get ballpark figure.
    let ns_single = ns_iter_inner(freq, inner, 1);

    // Try to estimate iter count for 1ms falling back to 1m
    // iterations if first run took < 1ns.
    let ns_target_total = 1_000_000; // 1ms
    let mut n = ns_target_total / cmp::max(1, ns_single);

    // if the first run took more than 1ms we don't want to just
    // be left doing 0 iterations on every loop. The unfortunate
    // side effect of not being able to do as many runs is
    // automatically handled by the statistical analysis below
    // (i.e., larger error bars).
    n = cmp::max(1, n);

    let mut total_run = Duration::new(0, 0);
    let samples: &mut [f64] = &mut [0.0_f64; 50];
    loop {
        let loop_start = Instant::now(freq);

        for p in &mut *samples {
            *p = ns_iter_inner(freq, inner, n) as f64 / n as f64;
        }

        stats::winsorize(samples, 5.0);
        let summ = stats::Summary::new(samples);

        for p in &mut *samples {
            let ns = ns_iter_inner(freq, inner, 5 * n);
            *p = ns as f64 / (5 * n) as f64;
        }

        stats::winsorize(samples, 5.0);
        let summ5 = stats::Summary::new(samples);

        let loop_run = loop_start.elapsed();

        // If we've run for 100ms and seem to have converged to a
        // stable median.
        if loop_run > Duration::from_millis(100)
            && summ.median_abs_dev_pct < 1.0
            && summ.median - summ5.median < summ5.median_abs_dev
        {
            return summ5;
        }

        total_run += loop_run;
        // Longest we ever run for is 3s.
        if total_run > Duration::from_secs(3) {
            return summ5;
        }

        // If we overflow here just return the results so far. We check a
        // multiplier of 10 because we're about to multiply by 2 and the
        // next iteration of the loop will also multiply by 5 (to calculate
        // the summ5 result)
        n = match n.checked_mul(10) {
            Some(_) => n * 2,
            None => {
                return summ5;
            }
        };
    }
}
