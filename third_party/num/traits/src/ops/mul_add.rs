/// Fused multiply-add. Computes `(self * a) + b` with only one rounding
/// error, yielding a more accurate result than an unfused multiply-add.
///
/// Using `mul_add` can be more performant than an unfused multiply-add if
/// the target architecture has a dedicated `fma` CPU instruction.
///
/// Note that `A` and `B` are `Self` by default, but this is not mandatory.
///
/// # Example
///
/// ```
/// use std::f32;
///
/// let m = 10.0_f32;
/// let x = 4.0_f32;
/// let b = 60.0_f32;
///
/// // 100.0
/// let abs_difference = (m.mul_add(x, b) - (m*x + b)).abs();
///
/// assert!(abs_difference <= f32::EPSILON);
/// ```
pub trait MulAdd<A = Self, B = Self> {
    /// The resulting type after applying the fused multiply-add.
    type Output;

    /// Performs the fused multiply-add operation.
    fn mul_add(self, a: A, b: B) -> Self::Output;
}

/// The fused multiply-add assignment operation.
pub trait MulAddAssign<A = Self, B = Self> {
    /// Performs the fused multiply-add operation.
    fn mul_add_assign(&mut self, a: A, b: B);
}

impl MulAdd<f32, f32> for f32 {
    type Output = Self;

    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self::Output {
        f32::mul_add(self, a, b)
    }
}

impl MulAdd<f64, f64> for f64 {
    type Output = Self;

    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self::Output {
        f64::mul_add(self, a, b)
    }
}

macro_rules! mul_add_impl {
    ($trait_name:ident for $($t:ty)*) => {$(
        impl $trait_name for $t {
            type Output = Self;

            #[inline]
            fn mul_add(self, a: Self, b: Self) -> Self::Output {
                (self * a) + b
            }
        }
    )*}
}

mul_add_impl!(MulAdd for isize usize i8 u8 i16 u16 i32 u32 i64 u64);
mul_add_impl!(MulAdd for i128 u128);

impl MulAddAssign<f32, f32> for f32 {
    #[inline]
    fn mul_add_assign(&mut self, a: Self, b: Self) {
        *self = f32::mul_add(*self, a, b)
    }
}

impl MulAddAssign<f64, f64> for f64 {
    #[inline]
    fn mul_add_assign(&mut self, a: Self, b: Self) {
        *self = f64::mul_add(*self, a, b)
    }
}

macro_rules! mul_add_assign_impl {
    ($trait_name:ident for $($t:ty)*) => {$(
        impl $trait_name for $t {
            #[inline]
            fn mul_add_assign(&mut self, a: Self, b: Self) {
                *self = (*self * a) + b
            }
        }
    )*}
}

mul_add_assign_impl!(MulAddAssign for isize usize i8 u8 i16 u16 i32 u32 i64 u64);
mul_add_assign_impl!(MulAddAssign for i128 u128);

