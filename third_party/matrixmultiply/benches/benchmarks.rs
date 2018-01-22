extern crate matrixmultiply;
pub use matrixmultiply::sgemm;
pub use matrixmultiply::dgemm;

#[macro_use]
extern crate bencher;


// Compute GFlop/s
// by flop / s = 2 M N K / time


macro_rules! mat_mul {
    ($modname:ident, $gemm:ident, $(($name:ident, $m:expr, $n:expr, $k:expr))+) => {
        mod $modname {
            use bencher::{Bencher};
            use $gemm;
            $(
            pub fn $name(bench: &mut Bencher)
            {
                let a = vec![0.; $m * $n]; 
                let b = vec![0.; $n * $k];
                let mut c = vec![0.; $m * $k];
                bench.iter(|| {
                    unsafe {
                        $gemm(
                            $m, $n, $k,
                            1.,
                            a.as_ptr(), $n, 1,
                            b.as_ptr(), $k, 1,
                            0.,
                            c.as_mut_ptr(), $k, 1,
                            )
                    }
                });
            }
            )+
        }
        benchmark_group!{ $modname, $($modname::$name),+ }
    };
}

benchmark_main!(mat_mul_f32, mat_mul_f64, ref_mat_mul_f32);

mat_mul!{mat_mul_f32, sgemm,
    (m004, 4, 4, 4)
    (m005, 5, 5, 5)
    (m006, 6, 6, 6)
    (m007, 7, 7, 7)
    (m008, 8, 8, 8)
    (m009, 9, 9, 9)
    (m012, 12, 12, 12)
    (m016, 16, 16, 16)
    (m032, 32, 32, 32)
    (m064, 64, 64, 64)
    (m127, 127, 127, 127)
    (m256, 256, 256, 256)
    (m512, 512, 512, 512)
    (mix16x4, 32, 4, 32)
    (mix32x2, 32, 2, 32)
    (mix97, 97, 97, 125)
    (mix128x10000x128, 128, 10000, 128)
}

mat_mul!{mat_mul_f64, dgemm,
    (m004, 4, 4, 4)
    (m007, 7, 7, 7)
    (m008, 8, 8, 8)
    (m012, 12, 12, 12)
    (m016, 16, 16, 16)
    (m032, 32, 32, 32)
    (m064, 64, 64, 64)
    (m127, 127, 127, 127)
    (m256, 256, 256, 256)
    (m512, 512, 512, 512)
    (mix16x4, 32, 4, 32)
    (mix32x2, 32, 2, 32)
    (mix97, 97, 97, 125)
    (mix128x10000x128, 128, 10000, 128)
}

use std::ops::{Add, Mul};

trait Z {
    fn zero() -> Self;
}
impl Z for f32 { fn zero() -> Self { 0. } }
impl Z for f64 { fn zero() -> Self { 0. } }

// simple, slow, correct (hopefully) mat mul (Row Major)
#[inline(never)]
fn reference_mat_mul<A>(m: usize, k: usize, n: usize, a: &[A], b: &[A], c: &mut [A])
    where A: Z + Add<Output=A> + Mul<Output=A> + Copy,
{
    assert!(a.len() >= m * k);
    assert!(b.len() >= k * n);
    assert!(c.len() >= m * n);

    for i in 0..m {
        for j in 0..n {
            unsafe {
                let celt = c.get_unchecked_mut(i * m + j);
                *celt = (0..k).fold(A::zero(),
                    move |s, x| s + *a.get_unchecked(i * k + x) * *b.get_unchecked(x * n + j));
            }
        }
    }
}

macro_rules! ref_mat_mul {
    ($modname:ident, $ty:ty, $(($name:ident, $m:expr, $n:expr, $k:expr))+) => {
        mod $modname {
            use bencher::{Bencher};
            use super::reference_mat_mul;
            $(
            pub fn $name(bench: &mut Bencher)
            {
                let a = vec![0. as $ty; $m * $n]; 
                let b = vec![0.; $n * $k];
                let mut c = vec![0.; $m * $k];
                bench.iter(|| {
                    reference_mat_mul($m, $n, $k, &a, &b, &mut c);
                    c[0]
                });
            }
            )+
        }
        benchmark_group!{ $modname, $($modname::$name),+ }
    };
}
ref_mat_mul!{ref_mat_mul_f32, f32,
    (m004, 4, 4, 4)
    (m005, 5, 5, 5)
    (m006, 6, 6, 6)
    (m007, 7, 7, 7)
    (m008, 8, 8, 8)
    (m009, 9, 9, 9)
    (m012, 12, 12, 12)
    (m016, 16, 16, 16)
    (m032, 32, 32, 32)
    (m064, 64, 64, 64)
}
