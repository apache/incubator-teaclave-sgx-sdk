use std::ops::{Mul, Add, Div, Sub, Rem,
               MulAssign, AddAssign, DivAssign, SubAssign, RemAssign,
               Neg, Not,
               BitAnd, BitOr, BitXor, BitAndAssign, BitOrAssign, BitXorAssign,
               Index, IndexMut};
use std::vec::*;
use utils;

use super::Vector;

/// Indexes vector.
impl<T> Index<usize> for Vector<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &T {
        assert!(idx < self.size);
        unsafe { self.data.get_unchecked(idx) }
    }
}

/// Indexes mutable vector.
impl<T> IndexMut<usize> for Vector<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        assert!(idx < self.size);
        unsafe { self.data.get_unchecked_mut(idx) }
    }
}

macro_rules! impl_bin_op_scalar_vector (
    ($trt:ident, $op:ident, $sym:tt, $doc:expr) => (

/// Scalar
#[doc=$doc]
/// with Vector reusing current memory.
impl<T: Copy + $trt<T, Output = T>> $trt<T> for Vector<T> {
    type Output = Vector<T>;
    fn $op(self, f: T) -> Vector<T> {
        self $sym &f
    }
}

/// Scalar
#[doc=$doc]
/// with Vector reusing current memory.
impl<'a, T: Copy + $trt<T, Output = T>> $trt<&'a T> for Vector<T> {
    type Output = Vector<T>;
    fn $op(mut self, f: &T) -> Vector<T> {
        for val in &mut self.data {
            *val = *val $sym *f;
        }
        self
    }
}

/// Scalar
#[doc=$doc]
/// with Vector allocating new memory.
impl<'a, T: Copy + $trt<T, Output = T>> $trt<T> for &'a Vector<T> {
    type Output = Vector<T>;
    fn $op(self, f: T) -> Vector<T> {
        self $sym &f
    }
}

/// Scalar
#[doc=$doc]
/// with Vector allocating new memory.
impl<'a, 'b, T: Copy + $trt<T, Output = T>> $trt<&'b T> for &'a Vector<T> {
    type Output = Vector<T>;
    fn $op(self, f: &T) -> Vector<T> {
        let new_data = self.data.iter().map(|v| *v $sym *f).collect();
        Vector { size: self.size, data: new_data }
    }
}
    );
);
impl_bin_op_scalar_vector!(Add, add, +, "addition");
impl_bin_op_scalar_vector!(Mul, mul, *, "multiplication");
impl_bin_op_scalar_vector!(Sub, sub, -, "subtraction");
impl_bin_op_scalar_vector!(Div, div, /, "division");
impl_bin_op_scalar_vector!(Rem, rem, %, "remainder");
impl_bin_op_scalar_vector!(BitAnd, bitand, &, "bitwise-and");
impl_bin_op_scalar_vector!(BitOr, bitor, |, "bitwise-or");
impl_bin_op_scalar_vector!(BitXor, bitxor, ^, "bitwise-xor");

macro_rules! impl_bin_op_vector (
    ($trt:ident, $op:ident, $sym:tt, $doc:expr) => (

/// Vector
#[doc=$doc]
/// with Vector reusing current memory.
impl<T: Copy + $trt<T, Output = T>> $trt<Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    fn $op(self, v: Vector<T>) -> Vector<T> {
        self $sym &v
    }
}

/// Vector
#[doc=$doc]
/// with Vector reusing current memory.
impl<'a, T: Copy + $trt<T, Output = T>> $trt<&'a Vector<T>> for Vector<T> {
    type Output = Vector<T>;

    fn $op(mut self, v: &Vector<T>) -> Vector<T> {
        utils::in_place_vec_bin_op(&mut self.data, &v.data, |x, &y| *x = *x $sym y);
        self
    }
}

/// Vector
#[doc=$doc]
/// with Vector reusing current memory.
impl<'a, T: Copy + $trt<T, Output = T>> $trt<Vector<T>> for &'a Vector<T> {
    type Output = Vector<T>;

    fn $op(self, mut v: Vector<T>) -> Vector<T> {
        utils::in_place_vec_bin_op(&mut v.data, &self.data, |y, &x| *y = x $sym *y);
        v
    }
}

/// Vector
#[doc=$doc]
/// with Vector allocating new memory.
impl<'a, 'b, T: Copy + $trt<T, Output = T>> $trt<&'b Vector<T>> for &'a Vector<T> {
    type Output = Vector<T>;

    fn $op(self, v: &Vector<T>) -> Vector<T> {
        assert!(self.size == v.size);
        let new_data = utils::vec_bin_op(&self.data, &v.data, |x, y| x $sym y);
        Vector {
            size: self.size,
            data: new_data,
        }
    }
}
    );
);
impl_bin_op_vector!(Add, add, +, "addition");
impl_bin_op_vector!(Sub, sub, -, "subtraction");
impl_bin_op_vector!(Rem, rem, %, "remainder");
impl_bin_op_vector!(BitAnd, bitand, &, "bitwise-and");
impl_bin_op_vector!(BitOr, bitor, |, "bitwise-or");
impl_bin_op_vector!(BitXor, bitxor, ^, "bitwise-xor");

macro_rules! impl_unary_op (
    ($trt:ident, $op:ident, $sym:tt, $doc:expr) => (

/// Gets
#[doc=$doc]
/// of vector.
impl<T: $trt<Output = T> + Copy> $trt for Vector<T> {
    type Output = Vector<T>;

    fn $op(mut self) -> Vector<T> {
        for val in &mut self.data {
            *val = $sym *val;
        }
        self
    }
}

/// Gets
#[doc=$doc]
/// of vector.
impl<'a, T: $trt<Output = T> + Copy> $trt for &'a Vector<T> {
    type Output = Vector<T>;

    fn $op(self) -> Vector<T> {
        let new_data = self.data.iter().map(|v| $sym *v).collect::<Vec<_>>();
        Vector::new(new_data)
    }
}
    );
);
impl_unary_op!(Neg, neg, -, "negative");
impl_unary_op!(Not, not, !, "not");


macro_rules! impl_op_assign_vec_scalar (
    ($assign_trt:ident, $trt:ident, $op:ident, $op_assign:ident, $doc:expr) => (

/// Performs
#[doc=$doc]
/// assignment between a vector and a scalar.
impl<T : Copy + $trt<T, Output=T>> $assign_trt<T> for Vector<T> {
    fn $op_assign(&mut self, _rhs: T) {
        for x in &mut self.data {
            *x = (*x).$op(_rhs)
        }
    }
}

/// Performs
#[doc=$doc]
/// assignment between a vector and a scalar.
impl<'a, T : Copy + $trt<T, Output=T>> $assign_trt<&'a T> for Vector<T> {
    fn $op_assign(&mut self, _rhs: &T) {
        for x in &mut self.data {
            *x = (*x).$op(*_rhs)
        }
    }
}
    );
);

impl_op_assign_vec_scalar!(AddAssign, Add, add, add_assign, "addition");
impl_op_assign_vec_scalar!(SubAssign, Sub, sub, sub_assign, "subtraction");
impl_op_assign_vec_scalar!(DivAssign, Div, div, div_assign, "division");
impl_op_assign_vec_scalar!(MulAssign, Mul, mul, mul_assign, "multiplication");
impl_op_assign_vec_scalar!(RemAssign, Rem, rem, rem_assign, "reminder");
impl_op_assign_vec_scalar!(BitAndAssign, BitAnd, bitand, bitand_assign, "bitwise-and");
impl_op_assign_vec_scalar!(BitOrAssign, BitOr, bitor, bitor_assign, "bitwise-or");
impl_op_assign_vec_scalar!(BitXorAssign, BitXor, bitxor, bitxor_assign, "bitwise-xor");

macro_rules! impl_op_assign_vec (
    ($assign_trt:ident, $trt:ident, $op:ident, $op_assign:ident, $doc:expr) => (

/// Performs elementwise
#[doc=$doc]
/// assignment between two vectors.
impl<T : Copy + $trt<T, Output=T>> $assign_trt<Vector<T>> for Vector<T> {
    fn $op_assign(&mut self, _rhs: Vector<T>) {
        utils::in_place_vec_bin_op(&mut self.data, &_rhs.data, |x, &y| {*x = (*x).$op(y) });
    }
}

/// Performs elementwise
#[doc=$doc]
/// assignment between two vectors.
impl<'a, T : Copy + $trt<T, Output=T>> $assign_trt<&'a Vector<T>> for Vector<T> {
    fn $op_assign(&mut self, _rhs: &Vector<T>) {
        utils::in_place_vec_bin_op(&mut self.data, &_rhs.data, |x, &y| {*x = (*x).$op(y) });
    }
}
    );
);

impl_op_assign_vec!(AddAssign, Add, add, add_assign, "addition");
impl_op_assign_vec!(SubAssign, Sub, sub, sub_assign, "subtraction");
impl_op_assign_vec!(RemAssign, Rem, rem, rem_assign, "remainder");
impl_op_assign_vec!(BitAndAssign, BitAnd, bitand, bitand_assign, "bitwise-and");
impl_op_assign_vec!(BitOrAssign, BitOr, bitor, bitor_assign, "bitwise-or");
impl_op_assign_vec!(BitXorAssign, BitXor, bitxor, bitxor_assign, "bitwise-xor");

#[cfg(test)]
mod tests {
    use super::super::Vector;

    // *****************************************************
    // Index
    // *****************************************************

    #[test]
    fn vector_index_mut() {
        let our_vec = vec![1., 2., 3., 4.];
        let mut our_vector = Vector::new(our_vec.clone());

        for i in 0..4 {
            our_vector[i] += 1.;
        }

        assert_eq!(our_vector, vector![2., 3., 4., 5.]);
    }

    // *****************************************************
    // Arithmetic Ops
    // *****************************************************

    #[test]
    fn vector_mul_f32_broadcast() {
        let a = vector![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = 3.0;

        let exp = vector![3.0, 6.0, 9.0, 12.0, 15.0, 18.0];

        // Allocating new memory
        let c = &a * &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a * b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() * &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a * b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_mul_int_broadcast() {
        let a = vector![1, 2, 3, 4, 5];
        let b = 2;

        let exp = vector![2, 4, 6, 8, 10];

        // Allocating new memory
        let c = &a * &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a * b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() * &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a * b;
        assert_eq!(c, exp);
    }

    // mul_xxx_elemwise is tested in impl_vec

    #[test]
    fn vector_div_f32_broadcast() {
        let a = vector![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = 3.0;

        let exp = vector![1. / 3., 2. / 3., 3. / 3., 4. / 3., 5. / 3., 6. / 3.];

        // Allocating new memory
        let c = &a / &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a / b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() / &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a / b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_div_int_broadcast() {
        let a = vector![1, 2, 3, 4, 5];
        let b = 2;

        let exp = vector![0, 1, 1, 2, 2];

        // Allocating new memory
        let c = &a / &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a / b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() / &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a / b;
        assert_eq!(c, exp);
    }

    // div_xxx_elemwise is tested in impl_vec

    #[test]
    fn vector_add_f32_broadcast() {
        let a = vector![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = 2.0;

        let exp = vector![3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

        // Allocating new memory
        let c = &a + &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a + b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() + &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a + b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_add_int_broadcast() {
        let a = vector![1, 2, 3, 4, 5];
        let b = 2;

        let exp = vector![3, 4, 5, 6, 7];

        // Allocating new memory
        let c = &a + &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a + b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() + &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a + b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_add_f32_elemwise() {
        let a = vector![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = vector![2.0, 3.0, 4.0, 5.0, 6.0, 7.0];

        let exp = vector![3.0, 5.0, 7.0, 9.0, 11.0, 13.0];

        // Allocating new memory
        let c = &a + &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a + b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() + &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a + b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_add_int_elemwise() {
        let a = vector![1, 2, 3, 4, 5, 6];
        let b = vector![2, 3, 4, 5, 6, 7];

        let exp = vector![3, 5, 7, 9, 11, 13];

        // Allocating new memory
        let c = &a + &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a + b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() + &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a + b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_sub_f32_broadcast() {
        let a = vector![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = 2.0;

        let exp = vector![-1.0, 0.0, 1.0, 2.0, 3.0, 4.0];

        // Allocating new memory
        let c = &a - &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a - b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() - &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a - b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_sub_int_broadcast() {
        let a = vector![1, 2, 3, 4, 5];
        let b = 2;

        let exp = vector![-1, 0, 1, 2, 3];

        // Allocating new memory
        let c = &a - &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a - b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() - &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a - b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_sub_f32_elemwise() {
        let a = vector![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = vector![2.0, 3.0, 4.0, 5.0, 6.0, 7.0];

        let exp = vector![-1.0, -1.0, -1.0, -1.0, -1.0, -1.0];

        // Allocating new memory
        let c = &a - &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a - b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() - &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a - b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_sub_int_elemwise() {
        let a = vector![10, 11, 12, 13, 14];
        let b = vector![2, 4, 6, 8, 10];

        let exp = vector![8, 7, 6, 5, 4];

        // Allocating new memory
        let c = &a - &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a - b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() - &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a - b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_rem_f32_broadcast() {
        let a = vector![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = 2.0;

        let exp = vector![1.0, 0.0, 1.0, 0.0, 1.0, 0.0];

        // Allocating new memory
        let c = &a % &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a % b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() % &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a % b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_rem_int_broadcast() {
        let a = vector![1, 2, 3, 4, 5];
        let b = 3;

        let exp = vector![1, 2, 0, 1, 2];

        // Allocating new memory
        let c = &a % &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a % b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() % &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a % b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_rem_f32_elemwise() {
        let a = vector![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = vector![3.0, 3.0, 3.0, 4.0, 4.0, 4.0];

        let exp = vector![1.0, 2.0, 0.0, 0.0, 1.0, 2.0];

        // Allocating new memory
        let c = &a % &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a % b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() % &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a % b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_rem_int_elemwise() {
        let a = vector![1, 2, 3, 4, 5];
        let b = vector![2, 2, 2, 3, 3];

        let exp = vector![1, 0, 1, 1, 2];

        // Allocating new memory
        let c = &a % &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a % b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() % &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a % b;
        assert_eq!(c, exp);
    }

    // *****************************************************
    // Arithmetic Assignments
    // *****************************************************

    #[test]
    fn vector_add_assign_int_broadcast() {
        let mut a = (0..9).collect::<Vector<_>>();

        let exp = (2..11).collect::<Vector<_>>();

        a += &2;
        assert_eq!(a, exp);

        let mut a = (0..9).collect::<Vector<_>>();

        a += 2;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_add_assign_int_elemwise() {
        let mut a = (0..9).collect::<Vector<_>>();
        let b = (0..9).collect::<Vector<_>>();

        let exp = (0..9).map(|x| 2 * x).collect::<Vector<_>>();

        a += &b;
        assert_eq!(a, exp);

        let mut a = (0..9).collect::<Vector<_>>();

        a += b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_sub_assign_int_broadcast() {
        let mut a = (0..9).collect::<Vector<_>>();

        let exp = (-2..7).collect::<Vector<_>>();

        a -= &2;
        assert_eq!(a, exp);

        let mut a = (0..9).collect::<Vector<i32>>();
        a -= 2;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_sub_assign_int_elemwise() {
        let mut a = vector![1, 2, 3, 4, 5];
        let b = vector![2, 2, 2, 3, 3];

        let exp = vector![-1, 0, 1, 1, 2];

        a -= &b;
        assert_eq!(a, exp);

        let mut a = vector![1, 2, 3, 4, 5];

        a -= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_div_assign_f32_broadcast() {
        let a_data = vec![1f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let exp = vector![0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5];

        let mut a = Vector::new(a_data.clone());

        a /= &2f32;
        assert_eq!(a, exp);

        let mut a = Vector::new(a_data.clone());
        a /= 2f32;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_mul_assign_f32_broadcast() {
        let a_data = vec![1f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let exp = vector![2f32, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0];
        let mut a = Vector::new(a_data.clone());

        a *= &2f32;
        assert_eq!(a, exp);

        let mut a = Vector::new(a_data.clone());
        a *= 2f32;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_rem_assign_int_broadcast() {
        let mut a = vector![1, 2, 3];

        let exp = vector![1, 2, 0];

        a %= &3;
        assert_eq!(a, exp);

        let mut a = vector![1, 2, 3];
        a %= 3;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_rem_assign_int_elemwise() {
        let mut a = vector![1, 2, 3, 4, 5];
        let b = vector![2, 2, 2, 3, 3];

        let exp = vector![1, 0, 1, 1, 2];

        a %= &b;
        assert_eq!(a, exp);

        let mut a = vector![1, 2, 3, 4, 5];

        a %= b;
        assert_eq!(a, exp);
    }

    // *****************************************************
    // Bitwise Ops
    // *****************************************************

    #[test]
    fn vector_bitand_int_broadcast() {
        let a = vector![1, 2, 3, 4, 5];
        let b = 2;

        let exp = vector![1 & 2, 2 & 2, 3 & 2, 4 & 2, 5 & 2];

        // Allocating new memory
        let c = &a & &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a & b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() & &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a & b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_bitand_bool_broadcast() {
        let a = vector![true, false, true];
        let b = true;

        let exp = vector![true, false, true];

        // Allocating new memory
        let c = &a & &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a & b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() & &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a & b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_bitand_int_elemwise() {
        let a = vector![1, 2, 3, 4, 5, 6];
        let b = vector![2, 3, 4, 5, 6, 7];

        let exp = vector![1 & 2, 2 & 3, 3 & 4, 4 & 5, 5 & 6, 6 & 7];

        // Allocating new memory
        let c = &a & &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a & b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() & &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a & b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_bitand_bool_elemwise() {
        let a = vector![true, true, false, false];
        let b = vector![true, false, true, false];

        let exp = vector![true, false, false, false];

        // Allocating new memory
        let c = &a & &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a & b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() & &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a & b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_bitor_int_broadcast() {
        let a = vector![1, 2, 3, 4, 5];
        let b = 2;

        let exp = vector![1 | 2, 2 | 2, 3 | 2, 4 | 2, 5 | 2];

        // Allocating new memory
        let c = &a | &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a | b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() | &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a | b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_bitor_bool_broadcast() {
        let a = vector![true, false, true];
        let b = true;

        let exp = vector![true, true, true];

        // Allocating new memory
        let c = &a | &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a | b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() | &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a | b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_bitor_int_elemwise() {
        let a = vector![1, 2, 3, 4, 5, 6];
        let b = vector![2, 3, 4, 5, 6, 7];

        let exp = vector![1 | 2, 2 | 3, 3 | 4, 4 | 5, 5 | 6, 6 | 7];

        // Allocating new memory
        let c = &a | &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a | b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() | &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a | b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_bitor_bool_elemwise() {
        let a = vector![true, true, false, false];
        let b = vector![true, false, true, false];

        let exp = vector![true, true, true, false];

        // Allocating new memory
        let c = &a | &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a | b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() | &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a | b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_bitxor_int_broadcast() {
        let a = vector![1, 2, 3, 4, 5];
        let b = 2;

        let exp = vector![1 ^ 2, 2 ^ 2, 3 ^ 2, 4 ^ 2, 5 ^ 2];

        // Allocating new memory
        let c = &a ^ &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a ^ b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() ^ &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a ^ b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_bitxor_bool_broadcast() {
        let a = vector![true, false, true];
        let b = true;

        let exp = vector![false, true, false];

        // Allocating new memory
        let c = &a ^ &b;
        assert_eq!(c, exp);

        // Allocating new memory
        let c = &a ^ b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() ^ &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a ^ b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_bitxor_int_elemwise() {
        let a = vector![1, 2, 3, 4, 5, 6];
        let b = vector![2, 3, 4, 5, 6, 7];

        let exp = vector![1 ^ 2, 2 ^ 3, 3 ^ 4, 4 ^ 5, 5 ^ 6, 6 ^ 7];

        // Allocating new memory
        let c = &a ^ &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a ^ b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() ^ &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a ^ b;
        assert_eq!(c, exp);
    }

    #[test]
    fn vector_bitxor_bool_elemwise() {
        let a = vector![true, true, false, false];
        let b = vector![true, false, true, false];

        let exp = vector![false, true, true, false];

        // Allocating new memory
        let c = &a ^ &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = &a ^ b.clone();
        assert_eq!(c, exp);

        // Reusing memory
        let c = a.clone() ^ &b;
        assert_eq!(c, exp);

        // Reusing memory
        let c = a ^ b;
        assert_eq!(c, exp);
    }

    // *****************************************************
    // Bitwise Assignments
    // *****************************************************

    #[test]
    fn vector_bitand_assign_int_broadcast() {
        let mut a = vector![1, 2, 3, 4, 5];
        let b = 2;

        let exp = vector![1 & 2, 2 & 2, 3 & 2, 4 & 2, 5 & 2];

        a &= &b;
        assert_eq!(a, exp);

        let mut a = vector![1, 2, 3, 4, 5];

        a &= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_bitand_assign_bool_broadcast() {
        let mut a = vector![true, true, false, false];
        let b = true;

        let exp = vector![true, true, false, false];

        a &= &b;
        assert_eq!(a, exp);

        let mut a = vector![true, true, false, false];

        a &= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_bitand_assign_int_elemwise() {
        let mut a = vector![1, 2, 3, 4, 5];
        let b = vector![2, 2, 2, 3, 3];

        let exp = vector![1 & 2, 2 & 2, 3 & 2, 4 & 3, 5 & 3];

        a &= &b;
        assert_eq!(a, exp);

        let mut a = vector![1, 2, 3, 4, 5];

        a &= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_bitand_assign_bool_elemwise() {
        let mut a = vector![true, true, false, false];
        let b = vector![true, false, true, false];

        let exp = vector![true, false, false, false];

        a &= &b;
        assert_eq!(a, exp);

        let mut a = vector![true, true, false, false];

        a &= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_bitor_assign_int_broadcast() {
        let mut a = vector![1, 2, 3, 4, 5];
        let b = 2;

        let exp = vector![1 | 2, 2 | 2, 3 | 2, 4 | 2, 5 | 2];

        a |= &b;
        assert_eq!(a, exp);

        let mut a = vector![1, 2, 3, 4, 5];

        a |= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_bitor_assign_bool_broadcast() {
        let mut a = vector![true, true, false, false];
        let b = true;

        let exp = vector![true, true, true, true];

        a |= &b;
        assert_eq!(a, exp);

        let mut a = vector![true, true, false, false];

        a |= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_bitor_assign_int_elemwise() {
        let mut a = vector![1, 2, 3, 4, 5];
        let b = vector![2, 2, 2, 3, 3];

        let exp = vector![1 | 2, 2 | 2, 3 | 2, 4 | 3, 5 | 3];

        a |= &b;
        assert_eq!(a, exp);

        let mut a = vector![1, 2, 3, 4, 5];

        a |= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_bitor_assign_bool_elemwise() {
        let mut a = vector![true, true, false, false];
        let b = vector![true, false, true, false];

        let exp = vector![true, true, true, false];

        a |= &b;
        assert_eq!(a, exp);

        let mut a = vector![true, true, false, false];

        a |= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_bitxor_assign_int_broadcast() {
        let mut a = vector![1, 2, 3, 4, 5];
        let b = 2;

        let exp = vector![1 ^ 2, 2 ^ 2, 3 ^ 2, 4 ^ 2, 5 ^ 2];

        a ^= &b;
        assert_eq!(a, exp);

        let mut a = vector![1, 2, 3, 4, 5];

        a ^= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_bitxor_assign_bool_broadcast() {
        let mut a = vector![true, true, false, false];
        let b = true;

        let exp = vector![false, false, true, true];

        a ^= &b;
        assert_eq!(a, exp);

        let mut a = vector![true, true, false, false];

        a ^= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_bitxor_assign_int_elemwise() {
        let mut a = vector![1, 2, 3, 4, 5];
        let b = vector![2, 2, 2, 3, 3];

        let exp = vector![1 ^ 2, 2 ^ 2, 3 ^ 2, 4 ^ 3, 5 ^ 3];

        a ^= &b;
        assert_eq!(a, exp);

        let mut a = vector![1, 2, 3, 4, 5];

        a ^= b;
        assert_eq!(a, exp);
    }

    #[test]
    fn vector_bitxor_assign_bool_elemwise() {
        let mut a = vector![true, true, false, false];
        let b = vector![true, false, true, false];

        let exp = vector![false, true, true, false];

        a ^= &b;
        assert_eq!(a, exp);

        let mut a = vector![true, true, false, false];

        a ^= b;
        assert_eq!(a, exp);
    }

    // *****************************************************
    // Unary Ops
    // *****************************************************

    #[test]
    fn vector_neg_f32() {
        let a = vector![1., 2., 3., 4., 5., 6.];
        let exp = vector![-1., -2., -3., -4., -5., -6.];

        assert_eq!(- &a, exp);
        assert_eq!(- a, exp);
    }

    #[test]
    fn vector_neg_int() {
        let a = vector![1, 2, 3, 4, 5, 6];
        let exp = vector![-1, -2, -3, -4, -5, -6];

        assert_eq!(- &a, exp);
        assert_eq!(- a, exp);
    }

    #[test]
    fn vector_not_int() {
        let a = vector![1, 2, 3, 4, 5, 6];
        let exp = vector![!1, !2, !3, !4, !5, !6];

        assert_eq!(!&a, exp);
        assert_eq!(!a, exp);
    }

    #[test]
    fn vector_not_bool() {
        let a = vector![true, false, true];
        let exp = vector![false, true, false];

        assert_eq!(!&a, exp);
        assert_eq!(!a, exp);
    }
}
