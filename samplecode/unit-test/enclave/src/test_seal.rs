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

use sgx_rand::*;
use sgx_tseal::*;
use sgx_types::marker::*;
use sgx_types::*;
use std::prelude::v1::*;

fn to_sealed_log<T: Copy + ContiguousMemory>(
    sealed_data: &SgxSealedData<T>,
    sealed_log: *mut u8,
    sealed_log_size: u32,
) -> Option<*mut sgx_sealed_data_t> {
    unsafe {
        sealed_data.to_raw_sealed_data_t(sealed_log as *mut sgx_sealed_data_t, sealed_log_size)
    }
}

fn from_sealed_log<'a, T: Copy + ContiguousMemory>(
    sealed_log: *mut u8,
    sealed_log_size: u32,
) -> Option<SgxSealedData<'a, T>> {
    unsafe {
        SgxSealedData::<T>::from_raw_sealed_data_t(
            sealed_log as *mut sgx_sealed_data_t,
            sealed_log_size,
        )
    }
}

pub fn test_seal_unseal() {
    #[derive(Copy, Clone, Default, Debug)]
    struct RandData {
        key: u32,
        rand: [u8; 16],
    }

    unsafe impl ContiguousMemory for RandData {}

    let mut data = RandData::default();
    data.key = 0x1234;
    let mut rand = StdRng::new().unwrap();
    rand.fill_bytes(&mut data.rand);

    let aad: [u8; 0] = [0_u8; 0];
    let sealed_data = SgxSealedData::<RandData>::seal_data(&aad, &data).unwrap();

    let mut sealed_log_arr: [u8; 2048] = [0; 2048];
    let sealed_log = sealed_log_arr.as_mut_ptr();
    let sealed_log_size: u32 = 2048;
    let opt = to_sealed_log(&sealed_data, sealed_log, sealed_log_size);
    assert_eq!(opt.is_some(), true);

    let sealed_data = from_sealed_log::<RandData>(sealed_log, sealed_log_size).unwrap();
    let unsealed_data = sealed_data.unseal_data().unwrap();
    let udata = unsealed_data.get_decrypt_txt();
    assert_eq!(data.key, udata.key);
    assert_eq!(data.rand, udata.rand);
}

pub fn test_number_sealing() {
    let data: u64 = 123456789;
    let aad: [u8; 0] = [0_u8; 0];
    let sealed_data =
        SgxSealedData::<u64>::seal_data(&aad, &data).expect("error while sealing u64");
    let unsealed_data = sealed_data
        .unseal_data()
        .expect("error while unsealing u64");
    assert_eq!(*unsealed_data.get_decrypt_txt(), data);
}

pub fn test_array_sealing() {
    let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let aad: [u8; 0] = [0_u8; 0];
    let sealed_data =
        SgxSealedData::<[u8]>::seal_data(&aad, &data).expect("error while sealing array");
    let unsealed_data = sealed_data
        .unseal_data()
        .expect("error while unsealing array");
    assert_eq!(unsealed_data.get_decrypt_txt(), data);
}

pub fn test_mac_aadata_number() {
    let aad_data: u64 = 123456789;
    let mmac = SgxMacAadata::<u64>::mac_aadata(&aad_data).expect("error while mac data");
    let unsealed_mac = mmac.unmac_aadata().expect("error when unmac data");
    let inner = Box::into_raw(unsealed_mac);
    let inner_val = unsafe { *(inner as *mut u64) };
    assert_eq!(inner_val, aad_data);
}

pub fn test_mac_aadata_slice() {
    use std::slice;
    let aad_data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mmac = SgxMacAadata::<[u8]>::mac_aadata(&aad_data).expect("error while mac data");
    let unsealed_mac = mmac.unmac_aadata().expect("error when unmac data");
    let inner = Box::into_raw(unsealed_mac);
    let inner_slice = unsafe { slice::from_raw_parts(inner as *mut u8, 10) };
    assert_eq!(inner_slice, aad_data);
}
