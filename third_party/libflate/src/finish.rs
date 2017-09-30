/// `Finish` is a type that represents a value which
/// may have an error occurred during the computation.
///
/// Logically, `Finish<T, E>` is equivalent to `Result<T, (T, E)>`.
#[derive(Debug, Default, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Finish<T, E> {
    value: T,
    error: Option<E>,
}
impl<T, E> Finish<T, E> {
    /// Makes a new instance.
    ///
    /// # Examples
    /// ```
    /// use libflate::Finish;
    ///
    /// // The result value of a succeeded computation
    /// let succeeded = Finish::new("value", None as Option<()>);
    /// assert_eq!(succeeded.into_result(), Ok("value"));
    ///
    /// // The result value of a failed computation
    /// let failed = Finish::new("value", Some("error"));
    /// assert_eq!(failed.into_result(), Err("error"));
    /// ```
    pub fn new(value: T, error: Option<E>) -> Self {
        Finish {
            value: value,
            error: error,
        }
    }

    /// Unwraps the instance.
    ///
    /// # Examples
    /// ```
    /// use libflate::Finish;
    ///
    /// let succeeded = Finish::new("value", None as Option<()>);
    /// assert_eq!(succeeded.unwrap(), ("value", None));
    ///
    /// let failed = Finish::new("value", Some("error"));
    /// assert_eq!(failed.unwrap(), ("value", Some("error")));
    /// ```
    pub fn unwrap(self) -> (T, Option<E>) {
        (self.value, self.error)
    }

    /// Converts from `Finish<T, E>` to `Result<T, E>`.
    ///
    /// # Examples
    /// ```
    /// use libflate::Finish;
    ///
    /// let succeeded = Finish::new("value", None as Option<()>);
    /// assert_eq!(succeeded.into_result(), Ok("value"));
    ///
    /// let failed = Finish::new("value", Some("error"));
    /// assert_eq!(failed.into_result(), Err("error"));
    /// ```
    pub fn into_result(self) -> Result<T, E> {
        if let Some(e) = self.error {
            Err(e)
        } else {
            Ok(self.value)
        }
    }

    /// Converts from `Finish<T, E>` to `Result<&T, &E>`.
    ///
    /// # Examples
    /// ```
    /// use libflate::Finish;
    ///
    /// let succeeded = Finish::new("value", None as Option<()>);
    /// assert_eq!(succeeded.as_result(), Ok(&"value"));
    ///
    /// let failed = Finish::new("value", Some("error"));
    /// assert_eq!(failed.as_result(), Err(&"error"));
    /// ```
    pub fn as_result(&self) -> Result<&T, &E> {
        if let Some(ref e) = self.error {
            Err(e)
        } else {
            Ok(&self.value)
        }
    }
}
