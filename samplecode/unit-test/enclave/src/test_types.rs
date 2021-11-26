use sgx_types::metadata::*;
use sgx_types::*;

use core::mem;

pub fn check_metadata_size() {
    assert_eq!(mem::size_of::<layout_group_t>(), 32);
    assert_eq!(mem::size_of::<layout_entry_t>(), 32);
    assert_eq!(mem::size_of::<layout_t>(), 32);
    assert_eq!(mem::size_of::<css_header_t>(), 128);
    assert_eq!(mem::size_of::<css_key_t>(), 772);
    assert_eq!(mem::size_of::<css_body_t>(), 128);
    assert_eq!(mem::size_of::<css_buffer_t>(), 780);
    assert_eq!(mem::size_of::<enclave_css_t>(), 1808);
    assert_eq!(mem::size_of::<metadata_t>(), METADATA_SIZE);
}

pub fn check_version() {
    //https://github.com/intel/linux-sgx/blob/master/common/inc/internal/metadata.h#L41
    let curr_version = 0x0000000200000004;
    assert_eq!(
        meta_data_make_version!(MAJOR_VERSION, MINOR_VERSION),
        curr_version
    );
    assert_eq!(
        major_version_of_metadata!(curr_version),
        MAJOR_VERSION as u64
    );
    assert_eq!(
        minor_version_of_metadata!(curr_version),
        MINOR_VERSION as u64
    );
}
