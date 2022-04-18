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

.file "cet.S"

.ifdef __CET__
.macro _CET_ENDBR
    endbr64
.endm

.macro _CET_NOTRACK
    notrack
.endm
.else
.macro _CET_ENDBR
.endm

.macro _CET_NOTRACK
.endm
.endif


.equ NT_GNU_PROPERTY_TYPE_0,            0x5
.equ GNU_PROPERTY_X86_FEATURE_1_AND,    0xc0000002
.equ SECTION_ALIGNMENT,                 0x3
.equ GNU_PROPERTY_X86_FEATURE_1_IBT,    0x1
.equ GNU_PROPERTY_X86_FEATURE_1_SHSTK,  0x2
.equ GNU_PROPERTY_X86_FEATURE_1_CET,    (GNU_PROPERTY_X86_FEATURE_1_IBT | GNU_PROPERTY_X86_FEATURE_1_SHSTK)

    .pushsection ".note.gnu.property", "a"
    .p2align SECTION_ALIGNMENT              /* section alignment */
    .long 1f - 0f                           /* name size (not including padding) */
    .long 4f - 1f                           /* desc size (not including padding) */
    .long NT_GNU_PROPERTY_TYPE_0            /* type NT_GNU_PROPERTY_TYPE_0*/
0:  .asciz "GNU"                            /* name */
1:  .long GNU_PROPERTY_X86_FEATURE_1_AND    /* pr_type */
    .long 3f - 2f                           /* pr_datasz */
2:  .long GNU_PROPERTY_X86_FEATURE_1_CET    /* cet feature bits*/
3:  .p2align SECTION_ALIGNMENT
4:
    .popsection