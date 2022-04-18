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

.file "everifyreport.S"

.global Leverifyreport2_inst
DECLARE_LOCAL_FUNC everifyreport2
    SE_PROLOG
    mov  $SE_EVERIFYREPORT2, %rax  /* EVERIFYREPORT2 leaf */
Leverifyreport2_inst:
    ENCLU
    jz   .Leverifyreport2_done     /* if EVERIFYREPORT2 error, ZF flag is set and error code is set to rax */
    xor  %rax, %rax
.Leverifyreport2_done:
    SE_EPILOG
