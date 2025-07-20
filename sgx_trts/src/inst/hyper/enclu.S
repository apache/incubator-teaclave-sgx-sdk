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

.file "enclu.S"

.equ HC_EENTER,     0x80000000
.equ HC_EEXIT,      0x80000001
.equ HC_ERESUME,    0x80000005
.equ HC_EGETKEY,    0x8000000B
.equ HC_EREPORT,    0x8000000C
.equ HC_EQUOTE,     0x8000000D

.macro ENCLU
    cmp     $SE_EEXIT, %rax
    jne     1f

    mov     $HC_EEXIT, %rax
    vmmcall
    ud2
1:
    push    %rdi
    push    %rsi
    push    %rdx
    push    %rcx
    push    %rbx
    push    %rax

    pop     %rdi
    pop     %rsi
    pop     %rdx
    pop     %rcx
    pop     %r8
    pop     %r9

.type se3,@function
.protected se3
    call   se3
.endm
