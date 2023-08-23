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

.file "aexnotify.S"

DECLARE_LOCAL_FUNC second_phase
    movq    %rsi, %rsp
    mov     $SE_EDECCSSA, %rax
    ENCLU                               /* DECCSSA */
    jmp     *%rdx

.macro DISABLE_AEXNOTIFY_ECALL
    mov     -3*SE_WORDSIZE(%rbp), %rax  /* cssa */
    cmp      $0, %rax
    jne     .Lrestore_untrust_stack

    mov     %gs:0, %rax             /* read td data */
    cmp     $0, %rax                /* check if td is initialized */
    je      .Lrestore_untrust_stack
    mov     first_ssa_gpr(%rax), %rdx
    movb    aex_notify(%rdx), %cl
    cmpb    $0, %cl
    je     .Lrestore_untrust_stack
    /*
     * clear aexnotify bit to disable aexnotify temporarily and
     * set td.aex_notify_flag to indicate aexnotify is enabled
     * before EEXIT
     */
    movb    $0, aex_notify(%rdx)
    movl    $0x1, aex_notify_flag(%rax)
.endm

.macro DISABLE_AEXNOTIFY_OCALL
    movb    aex_notify(%rdx), %cl
    cmpb    $0, %cl
    je     .Lrestore_outside_stack

    /*
     * clear aexnotify to disable aexnotify temporarily and
     * set td.aex_notify_flag to indicate aexnotify is enabled
     * before EEXIT
     */
    movb    $0, aex_notify(%rdx)
    movl    $0x1, aex_notify_flag(%rbx)
.endm
