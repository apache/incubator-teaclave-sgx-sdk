#[macro_use]
extern crate rulinalg;

pub mod mat;

#[cfg(feature = "io")]
pub mod io {
    mod csv;
}
