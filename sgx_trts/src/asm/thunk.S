/* Licensed to the Apache Software Foundation (ASF) under one
   or more contributor license agreements.  See the NOTICE file
   distributed with this work for additional information
   regarding copyright ownership.  The ASF licenses this file
   to you under the Apache License, Version 2.0 (the
   "License"); you may not use this file except in compliance
   with the License.  You may obtain a copy of the License at

      http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing,
   software distributed under the License is distributed on an
   "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
   KIND, either express or implied.  See the License for the
   specific language governing permissions and limitations
   under the License. */

.file "thunk.S"

.macro DEFINE_THUNK indirect_register, thunk_name
    .globl \thunk_name
    .hidden \thunk_name
    .type \thunk_name, @function
\thunk_name:
    .cfi_startproc
    jmp *%\indirect_register
    .cfi_endproc
.endm

.text
.globl __x86_return_thunk
.hidden __x86_return_thunk
.type __x86_return_thunk, @function

.section .nipx,"ax",@progbits
__x86_return_thunk:
    .cfi_startproc
    ret
    .cfi_endproc


DEFINE_THUNK rax, __x86_indirect_thunk_rax
DEFINE_THUNK rbx, __x86_indirect_thunk_rbx
DEFINE_THUNK rcx, __x86_indirect_thunk_rcx
DEFINE_THUNK rdx, __x86_indirect_thunk_rdx
DEFINE_THUNK rbp, __x86_indirect_thunk_rbp
DEFINE_THUNK rsi, __x86_indirect_thunk_rsi
DEFINE_THUNK rdi, __x86_indirect_thunk_rdi
DEFINE_THUNK r8, __x86_indirect_thunk_r8
DEFINE_THUNK r9, __x86_indirect_thunk_r9
DEFINE_THUNK r10, __x86_indirect_thunk_r10
DEFINE_THUNK r11, __x86_indirect_thunk_r11
DEFINE_THUNK r12, __x86_indirect_thunk_r12
DEFINE_THUNK r13, __x86_indirect_thunk_r13
DEFINE_THUNK r14, __x86_indirect_thunk_r14
DEFINE_THUNK r15, __x86_indirect_thunk_r15
