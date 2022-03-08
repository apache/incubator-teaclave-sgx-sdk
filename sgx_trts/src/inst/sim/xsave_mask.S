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
XSAVE_MASK_LOW:
    .int -1

.section .nipd,"aw"
XSAVE_MASK_HIGH:
    .int -1

.macro SET_XSAVE_MASK
    lea_pic XSAVE_MASK_LOW, %rax
    lea_pic XSAVE_MASK_HIGH, %rdx
    movl    (%rax), %eax
    movl    (%rdx), %edx
.endm

.section .text,"ax",@progbits

DECLARE_LOCAL_FUNC set_xsave_mask_low
    lea_pic XSAVE_MASK_LOW, %rax
    movl    %edi, (%rax)
    ret

DECLARE_LOCAL_FUNC set_xsave_mask_high
    lea_pic XSAVE_MASK_HIGH, %rax
    movl    %edi, (%rax)
    ret
