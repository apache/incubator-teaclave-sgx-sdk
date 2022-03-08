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

.file "lowlib.S"

.section .nipx,"ax",@progbits

DECLARE_LOCAL_FUNC get_bp
    mov %rbp, %rax
    ret

.equ reg_rax,   (SE_WORDSIZE * 0)
.equ reg_rbx,   (SE_WORDSIZE * 1)
.equ reg_rcx,   (SE_WORDSIZE * 2)
.equ reg_rdx,   (SE_WORDSIZE * 3)
.equ reg_rsi,   (SE_WORDSIZE * 4)
.equ reg_rdi,   (SE_WORDSIZE * 5)
.equ reg_rbp,   (SE_WORDSIZE * 6)
.equ reg_rsp,   (SE_WORDSIZE * 7)
.equ reg_rip,   (SE_WORDSIZE * 8)

DECLARE_LOCAL_FUNC load_regs
    mov	%rdi, %rdx
    mov	reg_rax(%rdx), %rax
    mov	reg_rbx(%rdx), %rbx
    mov	reg_rcx(%rdx), %rcx
    mov	reg_rsi(%rdx), %rsi
    mov	reg_rdi(%rdx), %rdi
    mov	reg_rbp(%rdx), %rbp
    mov	reg_rsp(%rdx), %rsp
    push reg_rip(%rdx)
    mov reg_rdx(%rdx), %rdx
    ret
