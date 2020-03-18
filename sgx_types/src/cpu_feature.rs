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

use crate::*;

//
// The processor is a generic IA32 CPU
//
pub const CPU_FEATURE_GENERIC_IA32          :uint64_t = 0x00000001;

//
// Floating point unit is on-chip.
//
pub const CPU_FEATURE_FPU                   :uint64_t = 0x00000002;

//
// Conditional mov instructions are supported.
//
pub const CPU_FEATURE_CMOV                  :uint64_t = 0x00000004;

//
// The processor supports the MMX technology instruction set extensions
// to Intel Architecture.
//
pub const CPU_FEATURE_MMX                   :uint64_t = 0x00000008;

//
// The FXSAVE and FXRSTOR instructions are supported for fast
// save and restore of the floating point context.
//
pub const CPU_FEATURE_FXSAVE                :uint64_t = 0x00000010;

//
// Indicates the processor supports the Streaming SIMD Extensions Instructions.
//
pub const CPU_FEATURE_SSE                   :uint64_t = 0x00000020;

//
// Indicates the processor supports the Streaming SIMD
// Extensions 2 Instructions.
//
pub const CPU_FEATURE_SSE2                  :uint64_t = 0x00000040;

//
// Indicates the processor supports the Streaming SIMD
// Extensions 3 Instructions. (PNI)
//
pub const CPU_FEATURE_SSE3                  :uint64_t = 0x00000080;

//
// The processor supports the Supplemental Streaming SIMD Extensions 3
// instructions. (MNI)
//
pub const CPU_FEATURE_SSSE3                 :uint64_t = 0x00000100;

//
// The processor supports the Streaming SIMD Extensions 4.1 instructions.(SNI)
//
pub const CPU_FEATURE_SSE4_1                :uint64_t = 0x00000200;

//
// The processor supports the Streaming SIMD Extensions 4.1 instructions.
// (NNI + STTNI)
//
pub const CPU_FEATURE_SSE4_2                :uint64_t = 0x00000400;

//
// The processor supports MOVBE instruction.
//
pub const CPU_FEATURE_MOVBE                 :uint64_t = 0x00000800;


//
// The processor supports POPCNT instruction.
//
pub const CPU_FEATURE_POPCNT                :uint64_t = 0x00001000;


//
// The processor supports PCLMULQDQ instruction.
//
pub const CPU_FEATURE_PCLMULQDQ             :uint64_t = 0x00002000;

//
// The processor supports instruction extension for encryption.
//
pub const CPU_FEATURE_AES                   :uint64_t = 0x00004000;

//
// The processor supports 16-bit floating-point conversions instructions.
//
pub const CPU_FEATURE_F16C                  :uint64_t = 0x00008000;

//
// The processor supports AVX instruction extension.
//
pub const CPU_FEATURE_AVX                   :uint64_t = 0x00010000;

//
// The processor supports RDRND (read random value) instruction.
//
pub const CPU_FEATURE_RDRND                 :uint64_t = 0x00020000;

//
// The processor supports FMA instructions.
//
pub const CPU_FEATURE_FMA                   :uint64_t = 0x00040000;

//
// The processor supports two groups of advanced bit manipulation extensions. - Haswell introduced, AVX2 related 
//
pub const CPU_FEATURE_BMI                   :uint64_t = 0x00080000;

//
// The processor supports LZCNT instruction (counts the number of leading zero
// bits). - Haswell introduced
//
pub const CPU_FEATURE_LZCNT                 :uint64_t = 0x00100000;

//
// The processor supports HLE extension (hardware lock elision). - Haswell introduced
//
pub const CPU_FEATURE_HLE                   :uint64_t = 0x00200000;

//
// The processor supports RTM extension (restricted transactional memory) - Haswell AVX2 related.
//
pub const CPU_FEATURE_RTM                   :uint64_t = 0x00400000;

//
// The processor supports AVX2 instruction extension.
//
pub const CPU_FEATURE_AVX2                  :uint64_t = 0x00800000;

//
// The processor supports AVX512 dword/qword instruction extension. 
//
pub const CPU_FEATURE_AVX512DQ              :uint64_t = 0x01000000;

//
// The processor supports the PTWRITE instruction.
//
pub const CPU_FEATURE_PTWRITE               :uint64_t = 0x02000000;

//
// KNC instruction set
//
pub const CPU_FEATURE_KNCNI                 :uint64_t = 0x04000000;

//
// AVX512 foundation instructions
//
pub const CPU_FEATURE_AVX512F               :uint64_t = 0x08000000;

//
// The processor supports uint add with OF or CF flags (ADOX, ADCX)
//
pub const CPU_FEATURE_ADX                   :uint64_t = 0x10000000;

//
// The processor supports RDSEED instruction.
//
pub const CPU_FEATURE_RDSEED                :uint64_t = 0x20000000;

// AVX512IFMA52: vpmadd52huq and vpmadd52luq
pub const CPU_FEATURE_AVX512IFMA52          :uint64_t = 0x40000000;

//
// The processor is a full inorder (Silverthorne) processor
// 
pub const CPU_FEATURE_F_INORDER             :uint64_t = 0x80000000;


// AVX512 exponential and reciprocal instructions
pub const CPU_FEATURE_AVX512ER              :uint64_t = 0x100000000;

// AVX512 prefetch instructions
pub const CPU_FEATURE_AVX512PF              :uint64_t = 0x200000000;

// AVX-512 conflict detection instructions
pub const CPU_FEATURE_AVX512CD              :uint64_t = 0x400000000;

// Secure Hash Algorithm instructions (SHA)
pub const CPU_FEATURE_SHA                   :uint64_t = 0x800000000;

// Memory Protection Extensions (MPX)
pub const CPU_FEATURE_MPX                   :uint64_t = 0x1000000000;

// AVX512BW - AVX512 byte/word vector instruction set
pub const CPU_FEATURE_AVX512BW              :uint64_t = 0x2000000000;

// AVX512VL - 128/256-bit vector support of AVX512 instructions
pub const CPU_FEATURE_AVX512VL              :uint64_t = 0x4000000000;

// AVX512VBMI:  vpermb, vpermi2b, vpermt2b and vpmultishiftqb
pub const CPU_FEATURE_AVX512VBMI            :uint64_t = 0x8000000000;

// AVX512_4FMAPS: Single Precision FMA for multivector(4 vector) operand.
pub const CPU_FEATURE_AVX512_4FMAPS         :uint64_t = 0x10000000000;

// AVX512_4VNNIW: Vector Neural Network Instructions for multivector(4 vector) operand with word elements.
pub const CPU_FEATURE_AVX512_4VNNIW         :uint64_t = 0x20000000000;

// AVX512_VPOPCNTDQ: 512-bit vector POPCNT instruction.
pub const CPU_FEATURE_AVX512_VPOPCNTDQ      :uint64_t = 0x40000000000;

// AVX512_BITALG: vector bit algebra in AVX512
pub const CPU_FEATURE_AVX512_BITALG         :uint64_t = 0x80000000000;

// AVX512_VBMI2: additional byte, word, dword and qword capabilities
pub const CPU_FEATURE_AVX512_VBMI2          :uint64_t = 0x100000000000;

// GFNI: Galois Field New Instructions.
pub const CPU_FEATURE_GFNI                  :uint64_t = 0x200000000000;

// VAES: vector AES instructions
pub const CPU_FEATURE_VAES                  :uint64_t = 0x400000000000;

// VPCLMULQDQ: Vector CLMUL instruction set.
pub const CPU_FEATURE_VPCLMULQDQ            :uint64_t = 0x800000000000;

// AVX512_VNNI: vector Neural Network Instructions.
pub const CPU_FEATURE_AVX512_VNNI           :uint64_t = 0x1000000000000;

// CLWB: Cache Line Write Back
pub const CPU_FEATURE_CLWB                  :uint64_t = 0x2000000000000;

// RDPID: Read Processor ID.
pub const CPU_FEATURE_RDPID                 :uint64_t = 0x4000000000000;

// IBT - Indirect branch tracking
pub const CPU_FEATURE_IBT                   :uint64_t = 0x8000000000000;

// Shadow stack
pub const CPU_FEATURE_SHSTK                 :uint64_t = 0x10000000000000;

// Intel Software Guard Extensions
pub const CPU_FEATURE_SGX                   :uint64_t = 0x20000000000000;

// Write back and do not invalidate cache
pub const CPU_FEATURE_WBNOINVD              :uint64_t = 0x40000000000000;

// Platform configuration - 1 << 55
pub const CPU_FEATURE_PCONFIG               :uint64_t = 0x80000000000000;


// Reserved feature bits
pub const RESERVED_CPU_FEATURE_BIT          :uint64_t = !(0x100000000000000 - 1);

// Incompatible bits which we should unset in trts
pub const INCOMPAT_FEATURE_BIT              :uint64_t = (1 << 11) | (1 << 12) | (1 << 25) | (1 << 26) | (1 << 27) | (1 << 28);


