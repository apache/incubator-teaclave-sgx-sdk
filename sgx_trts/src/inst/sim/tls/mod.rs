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

use crate::arch::Tcs;
use crate::inst::sim::TcsSim;

mod gnu;

impl TcsSim {
    #[link_section = ".nipx"]
    pub fn restore_td(&self) {
        let dtv = gnu::Dtv::get();
        dtv.set_value(self.saved_dtv);
        gnu::set_fs_gs_0(self.saved_fs_gs_0);
    }

    #[link_section = ".nipx"]
    pub fn set_td(&mut self, tcs: &Tcs, enclave_base: usize) {
        let dtv = gnu::Dtv::get();
        // save the old DTV[0].pointer->value
        self.saved_dtv = dtv.read_value();
        // save the old fs:0x0 or gs:0x0 value
        self.saved_fs_gs_0 = gnu::get_fs_gs_0();
        // set the DTV[0].pointer->value to TLS address
        let tib = enclave_base + tcs.ofsbase as usize;
        dtv.set_value(tib);

        //set the fs:0x0 or gs:0x0 to TLS address
        gnu::set_fs_gs_0(tib);
    }
}
