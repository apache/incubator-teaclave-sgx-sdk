use core::ffi::{c_int, c_void};
use sgx_new_edl::ocalls;
use sgx_types::{error::SgxStatus, types::timespec};

ocalls! {
    #[no_impl]
    pub fn u_read_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_pread64_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_write_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_pwrite64_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_sendfile_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_copy_file_range_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_splice_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_fcntl_arg0_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_fcntl_arg1_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_ioctl_arg0_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_ioctl_arg1_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_close_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_isatty_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_dup_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_eventfd_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_futimens_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_malloc_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_free_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_mmap_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_munmap_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_msync_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_mprotect_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_read_hostbuf_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_write_hostbuf_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_thread_set_event_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_thread_wait_event_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_thread_set_multiple_events_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_thread_setwait_events_ocall() -> SgxStatus;

    #[no_impl]
    pub fn u_clock_gettime_ocall() -> SgxStatus;
}
