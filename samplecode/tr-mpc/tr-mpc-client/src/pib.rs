use std::{mem,fmt,u16};

use hex;
use itertools::Itertools;

const SGX_CPUSVN_SIZE: usize = 16;
const PSVN_SIZE: usize = 18; // sizeof(psvn_t)
const PSDA_SVN_SIZE: usize = 4;
const ISVSVN_SIZE: usize = 2;
const SGX_PLATFORM_INFO_SIZE:usize = 101;

const QE_EPID_GROUP_REVOKED : u8                            = 0x01;
const PERF_REKEY_FOR_QE_EPID_GROUP_AVAILABLE : u8           = 0x02;
const QE_EPID_GROUP_OUT_OF_DATE : u8                        = 0x04;

const QUOTE_CPUSVN_OUT_OF_DATE : u16                        = 0x0001;
const QUOTE_ISVSVN_QE_OUT_OF_DATE : u16                     = 0x0002;
const QUOTE_ISVSVN_PCE_OUT_OF_DATE : u16                    = 0x0004;

const PSE_ISVSVN_OUT_OF_DATE : u16                          = 0x0001;
const EPID_GROUP_ID_BY_PS_HW_GID_REVOKED : u16              = 0x0002;
const SVN_FROM_PS_HW_SEC_INFO_OUT_OF_DATE : u16             = 0x0004;
const SIGRL_VER_FROM_PS_HW_SIG_RLVER_OUT_OF_DATE :u16       = 0x0008;
const PRIVRL_VER_FROM_PS_HW_PRV_KEY_RLVER_OUT_OF_DATE:u16   = 0x0010;

#[allow(non_camel_case_types)]
type sgx_isv_svn_t = u16;   // 2 bytes
#[allow(non_camel_case_types)]
type tcb_psvn_t = [u8;PSVN_SIZE];
#[allow(non_camel_case_types)]
type psda_svn_t = [u8;PSDA_SVN_SIZE];
#[allow(non_camel_case_types)]
type pse_isvsvn_t = [u8;ISVSVN_SIZE];

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
struct sgx_cpu_svn_t {      // 16 bytes
    svn : [u8;SGX_CPUSVN_SIZE],
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
struct psvn_t {              // 16 + 2
    cpu_svn : sgx_cpu_svn_t,
    isv_svn : sgx_isv_svn_t,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
struct sgx_ec256_signature_t {
    gx : [u8;32],
    gy : [u8;32],
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct platform_info_blob {
    sgx_epid_group_flags        : u8,
    sgx_tcb_evaluation_flags    : u16,
    pse_evaluation_flags        : u16,
    latest_equivalent_tcb_psvn  : tcb_psvn_t,
    latest_pse_isvsvn           : pse_isvsvn_t,
    latest_psda_svn             : psda_svn_t,
    xeid                        : u32,
    gid                         : u32,
    signature                   : sgx_ec256_signature_t,
}

macro_rules! test_and_print {
    ($f:ident, $x:expr, $y:ident) => {
        if $x & $y != 0 {
            writeln!($f, "\t\t{}", stringify!($y))?;
        }
    }
}

impl fmt::Display for platform_info_blob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Platform info is:")?;
        writeln!(f, "\tsgx_epid_group_fags = {:02X}", self.sgx_epid_group_flags)?;
        test_and_print!(f, self.sgx_epid_group_flags, QE_EPID_GROUP_REVOKED);
        test_and_print!(f, self.sgx_epid_group_flags, PERF_REKEY_FOR_QE_EPID_GROUP_AVAILABLE);
        test_and_print!(f, self.sgx_epid_group_flags, QE_EPID_GROUP_OUT_OF_DATE);
        let tcb_eflags = u16::from_be(self.sgx_tcb_evaluation_flags);
        writeln!(f, "\n\tsgx_tcb_evaluation_flags = {:04X}", tcb_eflags)?;
        test_and_print!(f, tcb_eflags, QUOTE_CPUSVN_OUT_OF_DATE);
        test_and_print!(f, tcb_eflags, QUOTE_ISVSVN_QE_OUT_OF_DATE);
        test_and_print!(f, tcb_eflags, QUOTE_ISVSVN_PCE_OUT_OF_DATE);
        let pse_eflags = u16::from_be(self.pse_evaluation_flags);
        writeln!(f, "\n\tpse_evaluation_flags = {:04X}", pse_eflags)?;
        test_and_print!(f, pse_eflags, PSE_ISVSVN_OUT_OF_DATE);
        test_and_print!(f, pse_eflags, EPID_GROUP_ID_BY_PS_HW_GID_REVOKED);
        test_and_print!(f, pse_eflags, SVN_FROM_PS_HW_SEC_INFO_OUT_OF_DATE);
        test_and_print!(f, pse_eflags, SIGRL_VER_FROM_PS_HW_SIG_RLVER_OUT_OF_DATE);
        test_and_print!(f, pse_eflags, PRIVRL_VER_FROM_PS_HW_PRV_KEY_RLVER_OUT_OF_DATE);
        writeln!(f,"\n\tlatest_equivalent_tcb_psvn: {:02X}", self.latest_equivalent_tcb_psvn.iter().format(""))?;
        writeln!(f,"\n\tlatest_pse_isvsvn: {:02X}", self.latest_pse_isvsvn.iter().format(""))?;
        writeln!(f,"\n\tlatest_psda_svn: {:02X}", self.latest_psda_svn.iter().format(""))?;
        writeln!(f,"\n\txeid: {:08X}", u32::from_be(self.xeid))?;
        writeln!(f,"\n\tgid: {:08X}", u32::from_be(self.gid))?;
        writeln!(f,"\n\tsignature:")?;
        writeln!(f,"\t\tgy: {:02X}", self.signature.gx.iter().format(""))?;
        writeln!(f,"\t\tgy: {:02X}", self.signature.gy.iter().format(""))?;
        writeln!(f)
    }
}

#[allow(non_camel_case_types)]
#[repr(packed)]
pub struct platform_info {
    #[allow(unused)]
    platform_info: [u8;SGX_PLATFORM_INFO_SIZE],
}

impl platform_info {
    pub fn from_str(pib_str : &str) -> platform_info_blob {
        let z = hex::decode(pib_str).unwrap();
        assert_eq!(z.len(), 105);

        // Remove the TSV header (undocumented)
        let pib_vec = z[4..].to_vec();
        let mut pib_array : [u8;101] = [0;101];
        pib_array.clone_from_slice(&pib_vec[..]);
        let pib : platform_info_blob = unsafe { mem::transmute(pib_array)};

        pib
    }
}
