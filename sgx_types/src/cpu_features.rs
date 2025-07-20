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
// under the License.

//
// The processor is a generic IA32 CPU
//
pub const CPU_FEATURE_GENERIC_IA32: u64 = 0x00000001;

//
// Floating point unit is on-chip.
//
pub const CPU_FEATURE_FPU: u64 = 0x00000002;

//
// Conditional mov instructions are supported.
//
pub const CPU_FEATURE_CMOV: u64 = 0x00000004;

//
// The processor supports the MMX technology instruction set extensions
// to Intel Architecture.
//
pub const CPU_FEATURE_MMX: u64 = 0x00000008;

//
// The FXSAVE and FXRSTOR instructions are supported for fast
// save and restore of the floating point context.
//
pub const CPU_FEATURE_FXSAVE: u64 = 0x00000010;

//
// Indicates the processor supports the Streaming SIMD Extensions Instructions.
//
pub const CPU_FEATURE_SSE: u64 = 0x00000020;

//
// Indicates the processor supports the Streaming SIMD
// Extensions 2 Instructions.
//
pub const CPU_FEATURE_SSE2: u64 = 0x00000040;

//
// Indicates the processor supports the Streaming SIMD
// Extensions 3 Instructions. (PNI)
//
pub const CPU_FEATURE_SSE3: u64 = 0x00000080;

//
// The processor supports the Supplemental Streaming SIMD Extensions 3
// instructions. (MNI)
//
pub const CPU_FEATURE_SSSE3: u64 = 0x00000100;

//
// The processor supports the Streaming SIMD Extensions 4.1 instructions.(SNI)
//
pub const CPU_FEATURE_SSE4_1: u64 = 0x00000200;

//
// The processor supports the Streaming SIMD Extensions 4.1 instructions.
// (NNI + STTNI)
//
pub const CPU_FEATURE_SSE4_2: u64 = 0x00000400;

//
// The processor supports MOVBE instruction.
//
pub const CPU_FEATURE_MOVBE: u64 = 0x00000800;

//
// The processor supports POPCNT instruction.
//
pub const CPU_FEATURE_POPCNT: u64 = 0x00001000;

//
// The processor supports PCLMULQDQ instruction.
//
pub const CPU_FEATURE_PCLMULQDQ: u64 = 0x00002000;

//
// The processor supports instruction extension for encryption.
//
pub const CPU_FEATURE_AES: u64 = 0x00004000;

//
// The processor supports 16-bit floating-point conversions instructions.
//
pub const CPU_FEATURE_F16C: u64 = 0x00008000;

//
// The processor supports AVX instruction extension.
//
pub const CPU_FEATURE_AVX: u64 = 0x00010000;

//
// The processor supports RDRND (read random value) instruction.
//
pub const CPU_FEATURE_RDRND: u64 = 0x00020000;

//
// The processor supports FMA instructions.
//
pub const CPU_FEATURE_FMA: u64 = 0x00040000;

//
// The processor supports two groups of advanced bit manipulation extensions. - Haswell introduced, AVX2 related
//
pub const CPU_FEATURE_BMI: u64 = 0x00080000;

//
// The processor supports LZCNT instruction (counts the number of leading zero
// bits). - Haswell introduced
//
pub const CPU_FEATURE_LZCNT: u64 = 0x00100000;

//
// The processor supports HLE extension (hardware lock elision). - Haswell introduced
//
pub const CPU_FEATURE_HLE: u64 = 0x00200000;

//
// The processor supports RTM extension (restricted transactional memory) - Haswell AVX2 related.
//
pub const CPU_FEATURE_RTM: u64 = 0x00400000;

//
// The processor supports AVX2 instruction extension.
//
pub const CPU_FEATURE_AVX2: u64 = 0x00800000;

//
// The processor supports AVX512 dword/qword instruction extension.
//
pub const CPU_FEATURE_AVX512DQ: u64 = 0x01000000;

//
// The processor supports the PTWRITE instruction.
//
pub const CPU_FEATURE_PTWRITE: u64 = 0x02000000;

//
// KNC instruction set
//
pub const CPU_FEATURE_KNCNI: u64 = 0x04000000;

//
// AVX512 foundation instructions
//
pub const CPU_FEATURE_AVX512F: u64 = 0x08000000;

//
// The processor supports uint add with OF or CF flags (ADOX, ADCX)
//
pub const CPU_FEATURE_ADX: u64 = 0x10000000;

//
// The processor supports RDSEED instruction.
//
pub const CPU_FEATURE_RDSEED: u64 = 0x20000000;

// AVX512IFMA52: vpmadd52huq and vpmadd52luq
pub const CPU_FEATURE_AVX512IFMA52: u64 = 0x40000000;

//
// The processor is a full inorder (Silverthorne) processor
//
pub const CPU_FEATURE_F_INORDER: u64 = 0x80000000;

// AVX512 exponential and reciprocal instructions
pub const CPU_FEATURE_AVX512ER: u64 = 0x100000000;

// AVX512 prefetch instructions
pub const CPU_FEATURE_AVX512PF: u64 = 0x200000000;

// AVX-512 conflict detection instructions
pub const CPU_FEATURE_AVX512CD: u64 = 0x400000000;

// Secure Hash Algorithm instructions (SHA)
pub const CPU_FEATURE_SHA: u64 = 0x800000000;

// Memory Protection Extensions (MPX)
pub const CPU_FEATURE_MPX: u64 = 0x1000000000;

// AVX512BW - AVX512 byte/word vector instruction set
pub const CPU_FEATURE_AVX512BW: u64 = 0x2000000000;

// AVX512VL - 128/256-bit vector support of AVX512 instructions
pub const CPU_FEATURE_AVX512VL: u64 = 0x4000000000;

// AVX512VBMI:  vpermb, vpermi2b, vpermt2b and vpmultishiftqb
pub const CPU_FEATURE_AVX512VBMI: u64 = 0x8000000000;

// AVX512_4FMAPS: Single Precision FMA for multivector(4 vector) operand.
pub const CPU_FEATURE_AVX512_4FMAPS: u64 = 0x10000000000;

// AVX512_4VNNIW: Vector Neural Network Instructions for multivector(4 vector) operand with word elements.
pub const CPU_FEATURE_AVX512_4VNNIW: u64 = 0x20000000000;

// AVX512_VPOPCNTDQ: 512-bit vector POPCNT instruction.
pub const CPU_FEATURE_AVX512_VPOPCNTDQ: u64 = 0x40000000000;

// AVX512_BITALG: vector bit algebra in AVX512
pub const CPU_FEATURE_AVX512_BITALG: u64 = 0x80000000000;

// AVX512_VBMI2: additional byte, word, dword and qword capabilities
pub const CPU_FEATURE_AVX512_VBMI2: u64 = 0x100000000000;

// GFNI: Galois Field New Instructions.
pub const CPU_FEATURE_GFNI: u64 = 0x200000000000;

// VAES: vector AES instructions
pub const CPU_FEATURE_VAES: u64 = 0x400000000000;

// VPCLMULQDQ: Vector CLMUL instruction set.
pub const CPU_FEATURE_VPCLMULQDQ: u64 = 0x800000000000;

// AVX512_VNNI: vector Neural Network Instructions.
pub const CPU_FEATURE_AVX512_VNNI: u64 = 0x1000000000000;

// CLWB: Cache Line Write Back
pub const CPU_FEATURE_CLWB: u64 = 0x2000000000000;

// RDPID: Read Processor ID.
pub const CPU_FEATURE_RDPID: u64 = 0x4000000000000;

// IBT - Indirect branch tracking
pub const CPU_FEATURE_IBT: u64 = 0x8000000000000;

// Shadow stack
pub const CPU_FEATURE_SHSTK: u64 = 0x10000000000000;

// Intel Software Guard Extensions
pub const CPU_FEATURE_SGX: u64 = 0x20000000000000;

// Write back and do not invalidate cache
pub const CPU_FEATURE_WBNOINVD: u64 = 0x40000000000000;

// Platform configuration - 1 << 55
pub const CPU_FEATURE_PCONFIG: u64 = 0x80000000000000;

// Reserved feature bits
pub const RESERVED_CPU_FEATURE_BIT: u64 = !(0x100000000000000 - 1);

// Incompatible bits which we should unset in trts
pub const INCOMPAT_FEATURE_BIT: u64 =
    (1 << 11) | (1 << 12) | (1 << 25) | (1 << 26) | (1 << 27) | (1 << 28);
