// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

use crate::enclave;
use sgx_types::impl_enum;

pub fn check_for(fid: Feature) -> bool {
    let bit = fid.get_feature_bit();
    (bit & enclave::rsgx_get_cpu_feature()) != 0
}

#[macro_export]
macro_rules! is_cpu_feature_supported {
    ($feature:expr) => {
        (($feature & $crate::enclave::rsgx_get_cpu_feature()) != 0)
    };
}

#[macro_export]
macro_rules! is_x86_feature_detected {
    ("ia32") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::ia32)
    };
    ("fpu") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::fpu)
    };
    ("cmov") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::cmov)
    };
    ("mmx") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::mmx)
    };
    ("fxsave") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::fxsave)
    };
    ("sse") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::sse)
    };
    ("sse2") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::sse2)
    };
    ("sse3") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::sse3)
    };
    ("ssse3") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::ssse3)
    };
    ("sse4.1") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::sse4_1)
    };
    ("sse4.2") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::sse4_2)
    };
    ("movbe") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::movbe)
    };
    ("popcnt") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::popcnt)
    };
    ("pclmulqdq") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::pclmulqdq)
    };
    ("aes") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::aes)
    };
    ("f16c") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::f16c)
    };
    ("avx") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx)
    };
    ("rdrand") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::rdrand)
    };
    ("fma") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::fma)
    };
    ("bmi") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::bmi)
    };
    ("lzcnt") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::lzcnt)
    };
    ("hle") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::hle)
    };
    ("rtm") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::rtm)
    };
    ("avx2") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx2)
    };
    ("avx512dq") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512dq)
    };
    ("ptwrite") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::ptwrite)
    };
    ("kncni") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::kncni)
    };
    ("avx512f") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512f)
    };
    ("adx") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::adx)
    };
    ("rdseed") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::rdseed)
    };
    ("avx512ifma") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512ifma)
    };
    ("inorder") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::full_inorder)
    };
    ("avx512er") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512er)
    };
    ("avx512pf") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512pf)
    };
    ("avx512cd") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512cd)
    };
    ("sha") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::sha)
    };
    ("mpx") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::mpx)
    };
    ("avx512bw") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512bw)
    };
    ("avx512vl") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512vl)
    };
    ("avx512vbmi") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512vbmi)
    };
    ("avx5124fmaps") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512_4fmaps)
    };
    ("avx5124vnniw") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512_4vnniw)
    };
    ("avx512vpopcntdq") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512_vpopcntdq)
    };
    ("avx512bitalg") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512_bitalg)
    };
    ("avx512vbmi2") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512vbmi2)
    };
    ("gfni") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::gfni)
    };
    ("vaes") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::vaes)
    };
    ("vpclmulqdq") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::vpclmulqdq)
    };
    ("avx512vnni") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::avx512vnni)
    };
    ("clwb") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::clwb)
    };
    ("rdpid") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::rdpid)
    };
    ("ibt") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::ibt)
    };
    ("shstk") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::shstk)
    };
    ("sgx") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::sgx)
    };
    ("wbnoinvd") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::wbnoinvd)
    };
    ("pconfig") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::pconfig)
    };
    ($t:tt,) => {
        is_x86_feature_detected!($t);
    };
    ($t:tt) => {
        compile_error!(concat!("unknown cpu feature: ", $t))
    };
}

impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum Feature {
        none            = 0,
        ia32            = 1,        /* 0x00000001 */
        fpu             = 2,        /* 0x00000002 */
        cmov            = 3,        /* 0x00000004 */
        mmx             = 4,        /* 0x00000008 */
        fxsave          = 5,        /* 0x00000010 */
        sse             = 6,        /* 0x00000020 */
        sse2            = 7,        /* 0x00000040 */
        sse3            = 8,        /* 0x00000080 */
        ssse3           = 9,        /* 0x00000100 */
        sse4_1          = 10,       /* 0x00000200 */
        sse4_2          = 11,       /* 0x00000400 */
        movbe           = 12,       /* 0x00000800 */
        popcnt          = 13,       /* 0x00001000 */
        pclmulqdq       = 14,       /* 0x00002000 */
        aes             = 15,       /* 0x00004000 */

        /* 16-bit floating-point conversions instructions */
        f16c            = 16,       /* 0x00008000 */
        /* AVX instructions - SNB */
        avx             = 17,       /* 0x00010000 */
        /* RDRND (read random value) instruction */
        rdrand          = 18,       /* 0x00020000 */
        /* FMA, may need more precise name - HSW */
        fma             = 19,       /* 0x00040000 */
        /* two groups of advanced bit manipulation extensions */
        bmi             = 20,       /* 0x00080000 */
        /* LZCNT (leading zeroes count) */
        lzcnt           = 21,       /* 0x00100000 */
        /* HLE (hardware lock elision) */
        hle             = 22,       /* 0x00200000 */
        /* RTM (restricted transactional memory) */
        rtm             = 23,       /* 0x00400000 */
        /* AVX2 instructions - HSW */
        avx2            = 24,       /* 0x00800000 */
        /* AVX512DQ - SKX 512-bit dword/qword vector instructions */
        avx512dq        = 25,       /* 0x01000000 */
        /* Unused, remained from KNF */
        ptwrite         = 26,       /* 0x02000000 */
        /* KNC new instructions */
        kncni           = 27,       /* 0x04000000 */
        /* AVX-512 foundation instructions - KNL and SKX */
        avx512f         = 28,       /* 0x08000000 */
        /* uint add with OF or CF flags (ADOX, ADCX) - BDW */
        adx             = 29,       /* 0x10000000 */
        /* Enhanced non-deterministic rand generator - BDW */
        rdseed          = 30,       /* 0x20000000 */
        /* AVX512IFMA52:  vpmadd52huq and vpmadd52luq. */
        avx512ifma      = 31,       /* 0x40000000 */
        /* Full inorder (like Silverthorne) processor */
        full_inorder    = 32,       /* 0x80000000 */
        /* AVX-512 exponential and reciprocal instructions - KNL */
        avx512er        = 33,       /* 0x100000000 */
        /* AVX-512 gather/scatter prefetch instructions - KNL */
        avx512pf        = 34,       /* 0x200000000 */
        /* AVX-512 conflict detection instructions - KNL */
        avx512cd        = 35,       /* 0x400000000 */
        /* Secure Hash Algorithm instructions (SHA) */
        sha             = 36,       /* 0x800000000 */
        /* Memory Protection Extensions (MPX) */
        mpx             = 37,       /* 0x1000000000 */
        /* AVX512BW - SKX 512-bit byte/word vector instructions */
        avx512bw        = 38,       /* 0x2000000000 */
        /* AVX512VL - adds 128/256-bit vector support of other AVX512 instructions. */
        avx512vl        = 39,       /* 0x4000000000 */
        /* AVX512VBMI:  vpermb, vpermi2b, vpermt2b and vpmultishiftqb. */
        avx512vbmi      = 40,       /* 0x8000000000 */
        /* AVX512_4FMAPS: Single Precision FMA for multivector(4 vector) operand. */
        avx512_4fmaps   = 41,       /* 0x10000000000 */
        /* AVX512_4VNNIW: Vector Neural Network Instructions for
        *  multivector(4 vector) operand with word elements. */
        avx512_4vnniw   = 42,       /* 0x20000000000 */
        /* AVX512_VPOPCNTDQ: 512-bit vector POPCNT. */
        avx512_vpopcntdq = 43,      /* 0x40000000000 */
        /* AVX512_BITALG: vector bit algebra in AVX512. */
        avx512_bitalg   = 44,       /* 0x80000000000 */
        /* AVX512_VBMI2: additional byte, word, dword and qword capabilities */
        avx512vbmi2     = 45,       /* 0x100000000000 */
        /* GFNI: Galois Field New Instructions. */
        gfni            = 46,       /* 0x200000000000 */
        /* VAES: vector AES instructions */
        vaes            = 47,       /* 0x400000000000 */
        /* VPCLMULQDQ: vector PCLMULQDQ instructions. */
        vpclmulqdq      = 48,       /* 0x800000000000 */
        /* AVX512_VNNI: vector Neural Network Instructions. */
        avx512vnni      = 49,       /* 0x1000000000000 */
        /* CLWB: Cache Line Write Back. */
        clwb            = 50,       /* 0x2000000000000 */
        /* RDPID: Read Processor ID. */
        rdpid           = 51,       /* 0x4000000000000 */
        ibt             = 52,       /* 0x8000000000000 */
        shstk           = 53,
        sgx             = 54,
        wbnoinvd        = 55,
        pconfig         = 56,
        end             = 57,
    }
}

impl Feature {
    pub fn get_feature_bit(&self) -> u64 {
        let id = *self as u32;
        if (id > Self::none as u32) && (id < Self::end as u32) {
            1 << (id - 1)
        } else {
            0
        }
    }
}
