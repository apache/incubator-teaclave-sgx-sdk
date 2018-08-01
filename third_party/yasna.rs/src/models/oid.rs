// Copyright 2016 Masaki Hara
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::{self,Display};
use std::prelude::v1::*;

/// A type that represents object identifiers.
///
/// This is actually a thin wrapper of `Vec<u64>`.
///
/// # Examples
///
/// ```
/// use yasna::models::ObjectIdentifier;
/// let sha384WithRSAEncryption = ObjectIdentifier::from_slice(&
///     [1, 2, 840, 113549, 1, 1, 12]);
/// println!("{}", sha384WithRSAEncryption);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ObjectIdentifier {
    components: Vec<u64>,
}

impl ObjectIdentifier {
    /// Constructs a new `ObjectIdentifier` from `Vec<u64>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yasna::models::ObjectIdentifier;
    /// let pkcs1 = ObjectIdentifier::new(
    ///     [1, 2, 840, 113549, 1, 1].to_vec());
    /// println!("{}", pkcs1);
    /// ```
    pub fn new(components: Vec<u64>) -> Self {
        return ObjectIdentifier {
            components: components,
        };
    }

    /// Constructs a new `ObjectIdentifier` from `&[u64]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yasna::models::ObjectIdentifier;
    /// let pkcs1 = ObjectIdentifier::from_slice(&
    ///     [1, 2, 840, 113549, 1, 1]);
    /// println!("{}", pkcs1);
    /// ```
    pub fn from_slice(components: &[u64]) -> Self {
        return ObjectIdentifier {
            components: components.to_vec(),
        };
    }

    /// Borrows its internal vector of components.
    ///
    /// # Examples
    ///
    /// ```
    /// use yasna::models::ObjectIdentifier;
    /// let pkcs1 = ObjectIdentifier::from_slice(&
    ///     [1, 2, 840, 113549, 1, 1]);
    /// let components : &Vec<u64> = pkcs1.components();
    /// ```
    pub fn components(&self) -> &Vec<u64> {
        &self.components
    }

    /// Mutably borrows its internal vector of components.
    ///
    /// # Examples
    ///
    /// ```
    /// use yasna::models::ObjectIdentifier;
    /// let mut pkcs1 = ObjectIdentifier::from_slice(&
    ///     [1, 2, 840, 113549, 1, 1]);
    /// let components : &mut Vec<u64> = pkcs1.components_mut();
    /// ```
    pub fn components_mut(&mut self) -> &mut Vec<u64> {
        &mut self.components
    }

    /// Extracts its internal vector of components.
    ///
    /// # Examples
    ///
    /// ```
    /// use yasna::models::ObjectIdentifier;
    /// let pkcs1 = ObjectIdentifier::from_slice(&
    ///     [1, 2, 840, 113549, 1, 1]);
    /// let mut components : Vec<u64> = pkcs1.into_components();
    /// ```
    pub fn into_components(self) -> Vec<u64> {
        self.components
    }
}

impl Display for ObjectIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut fst = true;
        for &component in &self.components {
            if fst {
                try!(write!(f, "{{{}", component));
            } else {
                try!(write!(f, " {}", component));
            }
            fst = false;
        }
        try!(write!(f, "}}"));
        return Ok(());
    }
}
