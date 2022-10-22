//! This is a fairly simple unpacked error representation that's used on
//! non-64bit targets, where the packed 64 bit representation wouldn't work, and
//! would have no benefit.

use super::{Custom, ErrorData, ErrorKind, SimpleMessage};
use alloc_crate::boxed::Box;

use sgx_types::sgx_status_t;

type Inner = ErrorData<Box<Custom>>;

pub(super) struct Repr(Inner);

impl Repr {
    #[inline]
    pub(super) fn new(dat: ErrorData<Box<Custom>>) -> Self {
        Self(dat)
    }
    pub(super) fn new_custom(b: Box<Custom>) -> Self {
        Self(Inner::Custom(b))
    }
    #[inline]
    pub(super) fn new_os(code: i32) -> Self {
        Self(Inner::Os(code))
    }
    #[inline]
    pub(super) fn new_simple(kind: ErrorKind) -> Self {
        Self(Inner::Simple(kind))
    }
    #[inline]
    pub(super) const fn new_simple_message(m: &'static SimpleMessage) -> Self {
        Self(Inner::SimpleMessage(m))
    }
    #[inline]
    pub(super) const fn new_sgx(status: sgx_status_t) -> Self {
        Self(Inner::SgxStatus(status))
    }
    #[inline]
    pub(super) fn into_data(self) -> ErrorData<Box<Custom>> {
        self.0
    }
    #[inline]
    pub(super) fn data(&self) -> ErrorData<&Custom> {
        match &self.0 {
            Inner::Os(c) => ErrorData::Os(*c),
            Inner::Simple(k) => ErrorData::Simple(*k),
            Inner::SimpleMessage(m) => ErrorData::SimpleMessage(*m),
            Inner::Custom(m) => ErrorData::Custom(&**m),
            Inner::SgxStatus(s) => ErrorData::SgxStatus(*s),
        }
    }
    #[inline]
    pub(super) fn data_mut(&mut self) -> ErrorData<&mut Custom> {
        match &mut self.0 {
            Inner::Os(c) => ErrorData::Os(*c),
            Inner::Simple(k) => ErrorData::Simple(*k),
            Inner::SimpleMessage(m) => ErrorData::SimpleMessage(*m),
            Inner::Custom(m) => ErrorData::Custom(&mut *m),
            Inner::SgxStatus(s) => ErrorData::SgxStatus(*s),
        }
    }
}
