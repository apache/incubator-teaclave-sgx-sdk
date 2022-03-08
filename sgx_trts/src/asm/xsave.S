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

.section .nipd,"aw"
XSAVE_ENABLED:
    .int 0

.macro DO_XSAVEC
    .byte   0x48, 0x0f, 0xc7, 0x21
.endm
.macro DO_XRSTOR
    .byte   0x48, 0x0f, 0xae, 0x29
.endm
.macro DO_FXSAVE
    .byte   0x48, 0x0f, 0xae, 0x01
.endm
.macro DO_FXRSTOR
    .byte   0x48, 0x0f, 0xae, 0x09
.endm

.section .nipx,"ax",@progbits

DECLARE_LOCAL_FUNC restore_xregs
    mov     %rdi, %rcx
    lea_pic XSAVE_ENABLED, %rax
    movl    (%rax), %eax
    cmpl    $0, %eax
    jz      1f
    SET_XSAVE_MASK
    DO_XRSTOR
    jmp     2f
1:
    DO_FXRSTOR
2:
    ret

DECLARE_LOCAL_FUNC save_xregs
    mov     %rdi, %rcx
    lea_pic XSAVE_ENABLED, %rax
    fwait
    movl    (%rax), %eax
    cmpl    $0, %eax
    jz      1f
    SET_XSAVE_MASK
    DO_XSAVEC
    jmp     2f
1:
    DO_FXSAVE
2:
    ret

.section .text,"ax",@progbits

DECLARE_LOCAL_FUNC get_xsave_enabled
    lea_pic XSAVE_ENABLED, %rcx
    xor     %rax, %rax
    movl    (%rcx), %eax
    ret

DECLARE_LOCAL_FUNC set_xsave_enabled
    lea_pic XSAVE_ENABLED, %rax
    movl    %edi, (%rax)
    ret
