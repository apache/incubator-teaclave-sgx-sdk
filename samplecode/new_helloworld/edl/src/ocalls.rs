use core::ffi::{c_int, c_void};
use sgx_new_edl::ocalls;
use sgx_types::{error::SgxStatus, types::timespec};

ocalls! {
    pub fn u_malloc_ocall(
        result: *mut *mut c_void,
        error: *mut c_int,
        size: usize,
        align: usize,
        zeroed: c_int,
    ) -> SgxStatus;

    pub fn u_thread_set_multiple_events_ocall(
        result: *mut i32,
        error: *mut i32,
        tcss: *const usize,
        total: usize,
    ) -> SgxStatus;

    pub fn u_thread_wait_event_ocall(
        result: *mut i32,
        error: *mut i32,
        tcs: usize,
        timeout: *const timespec,
        clockid: i32,
        absolute_time: i32,
    ) -> SgxStatus;

    pub fn bar() -> SgxStatus;
}
