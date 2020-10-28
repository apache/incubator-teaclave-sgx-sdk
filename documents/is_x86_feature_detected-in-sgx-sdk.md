---
permalink: /sgx-sdk-docs/is_x86_feature_detected-in-sgx-sdk
---

# `is_x86_feature_detected` in Teaclave SGX SDK

## Background

Crates often use `is_x86_feature_detected` to select appropriate implementations
(such as AVX/SSE/SSSE/FMA). It triggers `cpuid` instruction in default `libstd`
implementation on x86_64. We want to avoid such kind of SGX in-compatible
instructions and unnecessary AEX events.

## Solution

We found that Intel's SDK initializes its optimized libraries in a way of:

1. initialize a global cpu feature indicator by enclave initialization parameter
   in [urts](https://github.com/intel/linux-sgx/blob/042849cef8db1f0384e52e8cebcd8820c7754398/psw/urts/enclave_creator_hw_com.cpp#L61)

```c
//Since CPUID instruction is NOT supported within enclave, we enumerate the cpu features here and send to tRTS.
get_cpu_features(&info.cpu_features);
get_cpu_features_ext(&info.cpu_features_ext);
init_cpuinfo((uint32_t *)info.cpuinfo_table);
```

2. Initialize optimized libraries according to the global cpu feature indicator
   in [trts](https://github.com/intel/linux-sgx/blob/042849cef8db1f0384e52e8cebcd8820c7754398/sdk/trts/init_enclave.cpp#L169)

```c
// optimized libs
if (SDK_VERSION_2_0 < g_sdk_version || sys_features.size != 0)
{
  if (0 != init_optimized_libs(cpu_features, (uint32_t*)sys_features.cpuinfo_table, xfrm))
  {
    return -1;
  }
}
```

We found that in `init_optimized_libs`, a global variable
`g_cpu_feature_indicator` is initialized to store the `feature_bit_array` which
contains everything we need!

```c
static int set_global_feature_indicator(uint64_t feature_bit_array, uint64_t xfrm) {
    ......
    g_cpu_feature_indicator = feature_bit_array;
    return 0;
}
```

Since Rust SGX SDK depends on trts, we can simply re-use the
`g_cpu_feature_indicator` and simulate the `is_x86_feature_detected` macro
easily! First we import the value from trts:

```rust
#[link(name = "sgx_trts")]
extern {
    static g_cpu_feature_indicator: uint64_t;
    static EDMM_supported: c_int;
}

#[inline]
pub fn rsgx_get_cpu_feature() -> u64 {
    unsafe { g_cpu_feature_indicator }
}
```

Then parse `g_cpu_feature_indicator` like std_detect:

```rust
#[macro_export]
macro_rules! is_cpu_feature_supported {
    ($feature:expr) => ( (($feature & $crate::enclave::rsgx_get_cpu_feature()) != 0) )
}

#[macro_export]
macro_rules! is_x86_feature_detected {
    ("ia32") => {
        $crate::cpu_feature::check_for($crate::cpu_feature::Feature::ia32)
    };
    ...
}
```

## Performance concerns

We observed that some crates (such as matrixmultiply) are likely to use the
highest level of instructions for speed up. But it may not be the best solution.
For example, the "machine-learning" SGX sample depends on rusty-machine and
matrixmultiply, which intend to use AVX instruction if supported. However, if we
use the "fallback" mode, it'll be about 10x faster than the AVX version. The AVX
optimiztion is pretty complicated and I have no time to read Intel's [Intel® 64
and IA-32 Architectures Optimization Reference
Manual](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-optimization-manual.pdf).
And I don't think either of crate's owner or llvm backend can optimize it
ideally. I recommend to choose the appropirate instruction set per workload.
