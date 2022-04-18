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
   under the License.. */

.file "macro.S"

.equ SE_WORDSIZE,   8

.equ SE_EREPORT,        0
.equ SE_EGETKEY,        1
.equ SE_EENTER,         2
.equ SE_EEXIT,          4
.equ SE_EACCEPT,        5
.equ SE_EMODPE,         6
.equ SE_EVERIFYREPORT2, 8

.equ SE_ECREATE,    0
.equ SE_EADD,       1
.equ SE_EINIT,      2
.equ SE_EREMOVE,    3

.macro DECLARE_GLOBAL_FUNC name
    .globl \name
    .type \name, @function
\name:
    _CET_ENDBR
.endm

.macro DECLARE_LOCAL_FUNC name
    .globl \name
    .hidden \name
    .type \name, @function
\name:
    _CET_ENDBR
.endm

.macro SE_PROLOG
    .cfi_startproc
    push    %rbx
    push    %rcx
    push    %rdx
    movq    %rdi, %rbx
    movq    %rsi, %rcx
.endm

/*******************************************************************/

.macro SE_EPILOG
    pop     %rdx
    pop     %rcx
    pop     %rbx
    ret
    .cfi_endproc
.endm

.macro lea_pic symbol, reg
    lea   \symbol(%rip), \reg
.endm

.equ FLAGS_CLEAR_BITS, 0x40000
.macro CLEAN_XFLAGS
    pushfq
    notq     (%rsp)
    orq      $FLAGS_CLEAR_BITS, (%rsp)
    notq     (%rsp)
    popfq
.endm

.macro GET_STACK_BASE tcs
    mov      \tcs, %rax
    sub      $SE_GUARD_PAGE_SIZE, %rax
.endm
