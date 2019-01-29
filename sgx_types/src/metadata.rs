use types::{sgx_attributes_t, sgx_misc_select_t, sgx_measurement_t, sgx_isvext_prod_id_t, sgx_isvfamily_id_t};
use core::default::Default;

/* arch .h*/
pub const SE_KEY_SIZE       : usize =  384;
pub const SE_EXPONENT_SIZE  : usize =  4;

/* arch.h */
#[repr(packed)]
pub struct css_header_t {
    pub header          : [u8;12],
    pub css_type        : u32,  // type
    pub module_vendor   : u32,
    pub date            : u32,
    pub header2         : [u8;16],
    pub hw_version      : u32,
    pub reserved        : [u8;84],
}

#[repr(packed)]
pub struct css_key_t {
    pub modulus         : [u8;SE_KEY_SIZE],
    pub exponent        : [u8;SE_EXPONENT_SIZE],
    pub signature       : [u8;SE_KEY_SIZE],
}

#[repr(packed)]
pub struct css_body_t {
    pub misc_select     : sgx_misc_select_t,
    pub misc_mask       : sgx_misc_select_t,
    pub reserved        : [u8;4],
    pub isv_family_id   : sgx_isvfamily_id_t,
    pub attributes      : sgx_attributes_t,
    pub attribute_mask  : sgx_attributes_t,
    pub enclave_hash    : sgx_measurement_t,
    pub reserved2       : [u8;16],
    pub isvext_prod_id  : sgx_isvext_prod_id_t,
    pub isv_prod_id     : u16,
    pub isv_svn         : u16,
}

#[repr(packed)]
pub struct css_buffer_t {
    pub reserved        : [u8;12],
    pub q1              : [u8;SE_KEY_SIZE],
    pub q2              : [u8;SE_KEY_SIZE],
}

#[repr(packed)]
pub struct enclave_css_t {
    pub header          : css_header_t,
    pub key             : css_key_t,
    pub body            : css_body_t,
    pub buffer          : css_buffer_t,
}

/* version of metadata */
/* based on 2.4 */
pub const MAJOR_VERSION         : u32 = 2;
pub const MINOR_VERSION         : u32 = 4;
pub const SGX_2_1_MAJOR_VERSION : u32 = 2;   //MAJOR_VERSION should not larger than 0ffffffff
pub const SGX_2_1_MINOR_VERSION : u32 = 2;   //MINOR_VERSION should not larger than 0ffffffff
pub const SGX_2_0_MAJOR_VERSION : u32 = 2;   //MAJOR_VERSION should not larger than 0ffffffff
pub const SGX_2_0_MINOR_VERSION : u32 = 1;   //MINOR_VERSION should not larger than 0ffffffff
pub const SGX_1_9_MAJOR_VERSION : u32 = 1;   //MAJOR_VERSION should not larger than 0ffffffff
pub const SGX_1_9_MINOR_VERSION : u32 = 4;   //MINOR_VERSION should not larger than 0ffffffff
pub const SGX_1_5_MAJOR_VERSION : u32 = 1;   //MAJOR_VERSION should not larger than 0ffffffff
pub const SGX_1_5_MINOR_VERSION : u32 = 3;   //MINOR_VERSION should not larger than 0ffffffff

pub const METADATA_MAGIC        : u64 = 0x86A80294635D0E4C;
pub const METADATA_SIZE         : usize = 0x5000;
pub const TCS_TEMPLATE_SIZE     : usize = 72;

pub const TCS_POLICY_BIND       : u32 = 0x00000000;  /* If set, the TCS is bound to the application thread */
pub const TCS_POLICY_UNBIND     : u32 = 0x00000001;

pub const MAX_SAVE_BUF_SIZE     : u32 = 2632;
pub const TCS_NUM_MIN           : u32 = 1;
pub const SSA_NUM_MIN           : u32 = 2;
pub const SSA_FRAME_SIZE_MIN    : u32 = 1;
pub const SSA_FRAME_SIZE_MAX    : u32 = 2;
pub const STACK_SIZE_MIN        : u32 = 0x2000;
pub const STACK_SIZE_MAX        : u32 = 0x40000;
pub const HEAP_SIZE_MIN         : u32 = 0x1000;
pub const HEAP_SIZE_MAX         : u32 = 0x1000000;
pub const DEFAULT_MISC_SELECT   : u32 = 0;
pub const DEFAULT_MISC_MASK     : u32 = 0xFFFFFFFF;
pub const ISVFAMILYID_MAX       : u64 = 0xFFFFFFFFFFFFFFFF;
pub const ISVEXTPRODID_MAX      : u64 = 0xFFFFFFFFFFFFFFFF;

pub const STATIC_STACK_SIZE     : usize = 688;
pub const SE_GUARD_PAGE_SHIFT   : usize = 16;
pub const SE_GUARD_PAGE_SIZE    : usize = (1 << SE_GUARD_PAGE_SHIFT);

impl_struct! {
    #[repr(packed)]
    pub struct data_directory_t {
        pub offset : u32,
        pub size   : u32,
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum dir_index_t {
        DIR_PATCH  = 0,
        DIR_LAYOUT = 1,
        DIR_NUM    = 2,
    }
}

pub const GROUP_FLAG              :u32 = (1<<12);
pub const LAYOUT_ID_HEAP_MIN      :u32 = 1;
pub const LAYOUT_ID_HEAP_INIT     :u32 = 2;
pub const LAYOUT_ID_HEAP_MAX      :u32 = 3;
pub const LAYOUT_ID_TCS           :u32 = 4;
pub const LAYOUT_ID_TD            :u32 = 5;
pub const LAYOUT_ID_SSA           :u32 = 6;
pub const LAYOUT_ID_STACK_MAX     :u32 = 7;
pub const LAYOUT_ID_STACK_MIN     :u32 = 8;
pub const LAYOUT_ID_THREAD_GROUP  :u32 = group_id!(9);
pub const LAYOUT_ID_GUARD         :u32 = 10;
pub const LAYOUT_ID_HEAP_DYN_MIN  :u32 = 11;
pub const LAYOUT_ID_HEAP_DYN_INIT :u32 = 12;
pub const LAYOUT_ID_HEAP_DYN_MAX  :u32 = 13;
pub const LAYOUT_ID_TCS_DYN       :u32 = 14;
pub const LAYOUT_ID_TD_DYN        :u32 = 15;
pub const LAYOUT_ID_SSA_DYN       :u32 = 16;
pub const LAYOUT_ID_STACK_DYN_MAX :u32 = 17;
pub const LAYOUT_ID_STACK_DYN_MIN :u32 = 18;
pub const LAYOUT_ID_THREAD_GROUP_DYN :u32 = group_id!(19);

type si_flags_t = u64;

impl_struct! {
    #[repr(packed)]
    pub struct layout_entry_t {
        pub id             : u16,
        pub attributes     : u16,
        pub page_count     : u32,
        pub rva            : u64,
        pub content_size   : u32,
        pub content_offset : u32,
        pub si_flags       : si_flags_t,
    }
    
    #[repr(packed)]
    pub struct layout_group_t {
        pub id             : u16,
        pub entry_count    : u16,
        pub load_times     : u32,
        pub load_step      : u64,
        pub reserved       : [u32;4],
    }
}

#[allow(unused)]
#[repr(C,packed)]
pub union layout_t {
    entry : layout_entry_t,
    group : layout_group_t,
}

#[repr(packed)]
pub struct patch_entry_t {
    pub dst     : u64,
    pub src     : u32,
    pub size    : u32,
    pub reserved: [u32;4],
}

#[repr(packed)]
pub struct metadata_t {
    pub magic_num       : u64,
    pub version         : u64,
    pub size            : u32,
    pub tcs_policy      : u32,
    pub ssa_frame_size  : u32,
    pub max_save_buffer_size : u32,
    pub desired_misc_select: u32,
    pub tcs_min_pool    : u32,
    pub enclave_size    : u64,
    pub attributes      : sgx_attributes_t,
    pub enclave_css     : enclave_css_t,
    pub dirs            : [data_directory_t;dir_index_t::DIR_NUM as usize],
    pub data            : [u8;18592],
}
