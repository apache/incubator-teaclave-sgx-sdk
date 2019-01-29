//! Functions for calculating mean
use std::f64::NAN;

/// Calculates arithmetic mean (AM) of data set `slice`.
///
/// # Arguments
///
/// * `slice` - collection of values
///
/// # Example
///
/// ```
/// use math::mean;
///
/// let slice = [8., 16.];
/// assert_eq!(mean::arithmetic(&slice), 12.);
/// ```
pub fn arithmetic(slice: &[f64]) -> f64 {
	slice.iter().fold(0., |a, b| a + b) / slice.len() as f64
}

/// Calculate geometric mean (GM) of data set `slice`.
///
/// If the result would be imaginary, function returns `NAN`.
///
/// # Arguments
///
/// * `slice` - collection of values
///
/// # Example
///
/// ```
/// use math::mean;
///
/// let slice = [9., 16.];
/// assert_eq!(mean::geometric(&slice), 12.);
/// ```
pub fn geometric(slice: &[f64]) -> f64 {
	let product = slice.iter().fold(1., |a, b| a * b);
	match product < 0. {
		true => NAN,
		false => product.powf(1. / slice.len() as f64),
	}
}

/// Calculate harmonic mean (HM) of data set `slice`.
///
/// # Arguments
///
/// * `slice` - collection of values
///
/// # Example
///
/// ```
/// use math::mean;
///
/// let slice = [1., 7.];
/// assert_eq!(mean::harmonic(&slice), 1.75);
/// ```
pub fn harmonic(slice: &[f64]) -> f64 {
	slice.len() as f64 / slice.iter().fold(0., |a, b| a + 1. / b)
}

#[cfg(test)]
mod tests {
	use std::f64::{ NAN, INFINITY, NEG_INFINITY };
	use round;

	macro_rules! test_mean {
		($func:path [ $($name:ident: $params:expr,)* ]) => {
		$(
			#[test]
			fn $name() {
				let (slice, expected): (&[f64], f64) = $params;
				let result = $func(slice);
				match result.is_nan() {
					true => assert_eq!(expected.is_nan(), true),
					false => assert_eq!(round::half_up(result, 6), expected),
				}
			}
		)*
		}
	}

	test_mean! { super::arithmetic [
		arithmetic_1: (&[-7., -4., 1., 3., 8.], 0.2),
		arithmetic_2: (&[-4., 1., 3., 8., 12.], 4.),
		arithmetic_3: (&[0., 0., 0., 0., 0.], 0.),
		arithmetic_4: (&[0., 4., 7., 9., 17.], 7.4),
		arithmetic_5: (&[1., 2., 6., 4., 13.], 5.2),
		arithmetic_6: (&[1., 5., 10., 20., 25.], 12.2),
		arithmetic_7: (&[2., 3., 5., 7., 11.], 5.6),
		arithmetic_8: (&[NEG_INFINITY, 1., 2., 3., 4.], NEG_INFINITY),
		arithmetic_9: (&[1., 2., 3., 4., INFINITY], INFINITY),
	]}

	test_mean! { super::geometric [
		geometric_1: (&[-7., -4., 1., 3., 8.], 3.676833),
		geometric_2: (&[-4., 1., 3., 8., 12.], NAN),
		geometric_3: (&[0., 0., 0., 0., 0.], 0.),
		geometric_4: (&[0., 4., 7., 9., 17.], 0.),
		geometric_5: (&[1., 2., 6., 4., 13.], 3.622738),
		geometric_6: (&[1., 5., 10., 20., 25.], 7.578583),
		geometric_7: (&[2., 3., 5., 7., 11.], 4.706764),
		geometric_8: (&[NEG_INFINITY, 1., 2., 3., 4.], NAN),
		geometric_9: (&[1., 2., 3., 4., INFINITY], INFINITY),
	]}

	test_mean! { super::harmonic [
		harmonic_1: (&[-7., -4., 1., 3., 8.], 4.692737),
		harmonic_2: (&[-4., 1., 3., 8., 12.], 3.870968),
		harmonic_3: (&[0., 0., 0., 0., 0.], 0.),
		harmonic_4: (&[0., 4., 7., 9., 17.], 0.),
		harmonic_5: (&[1., 2., 6., 4., 13.], 2.508039),
		harmonic_6: (&[1., 5., 10., 20., 25.], 3.597122),
		harmonic_7: (&[2., 3., 5., 7., 11.], 3.94602),
		harmonic_8: (&[NEG_INFINITY, 1., 2., 3., 4.], 2.4),
		harmonic_9: (&[1., 2., 3., 4., INFINITY], 2.4),
	]}
}
