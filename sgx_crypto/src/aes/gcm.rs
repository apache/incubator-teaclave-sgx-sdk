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
// under the License.

use core::array::TryFromSliceError;
use core::convert::{From, TryFrom, TryInto};
use core::ptr;
use sgx_crypto_sys::*;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::{BytewiseEquality, ContiguousMemory};
use sgx_types::memeq::ConstTimeEq;
use sgx_types::types::{
    AesHandle, AlignKey128bit, AlignMac128bit, Key128bit, Mac128bit, AESGCM_IV_SIZE,
};

#[derive(Debug)]
pub struct AesGcm<A: AsRef<[u8]>> {
    key: AlignKey128bit,
    iv: Nonce,
    aad: Aad<A>,
    handle: AesHandle,
}

impl<A: AsRef<[u8]>> AesGcm<A> {
    pub fn new(key: &Key128bit, iv: Nonce, aad: Aad<A>) -> SgxResult<AesGcm<A>> {
        ensure!(
            aad.as_ref().len() < i32::MAX as usize,
            SgxStatus::InvalidParameter
        );

        Ok(AesGcm {
            key: AlignKey128bit::from(key),
            iv,
            aad,
            handle: ptr::null_mut(),
        })
    }

    pub fn encrypt(&mut self, src: &[u8], dst: &mut [u8]) -> SgxResult<Mac128bit> {
        let src_len = src.len();
        let dst_len = dst.len();
        let aad = self.aad.as_ref();
        let aad_len = aad.len();

        ensure!(src_len < i32::MAX as usize, SgxStatus::InvalidParameter);
        ensure!(src_len == dst_len, SgxStatus::InvalidParameter);

        let p_aad = if !aad.is_empty() {
            aad.as_ptr()
        } else {
            ptr::null()
        };

        let (p_src, p_dst) = if !src.is_empty() {
            (src.as_ptr(), dst.as_mut_ptr())
        } else {
            (ptr::null(), ptr::null_mut())
        };

        let mut mac = AlignMac128bit::default();
        let status = unsafe {
            sgx_rijndael128GCM_encrypt(
                &self.key.key as *const Key128bit,
                p_src,
                src_len as u32,
                p_dst,
                self.iv.as_ref().as_ptr(),
                self.iv.as_ref().len() as u32,
                p_aad,
                aad_len as u32,
                &mut mac.mac as *mut Mac128bit,
            )
        };

        ensure!(status.is_success(), status);
        Ok(mac.mac)
    }

    pub fn encrypt_in_place(&mut self, in_out: &mut [u8]) -> SgxResult<Mac128bit> {
        let mut dst = vec![0_u8; in_out.len()];
        let mac = self.encrypt(in_out, dst.as_mut_slice())?;
        in_out.clone_from_slice(dst.as_slice());
        Ok(mac)
    }

    pub fn decrypt(&mut self, src: &[u8], dst: &mut [u8], mac: &Mac128bit) -> SgxResult {
        let src_len = src.len();
        let dst_len = dst.len();
        let aad = self.aad.as_ref();
        let aad_len = aad.len();

        ensure!(src_len < i32::MAX as usize, SgxStatus::InvalidParameter);
        ensure!(src_len == dst_len, SgxStatus::InvalidParameter);

        let p_aad = if !aad.is_empty() {
            aad.as_ptr()
        } else {
            ptr::null()
        };

        let (p_src, p_dst) = if !src.is_empty() {
            (src.as_ptr(), dst.as_mut_ptr())
        } else {
            (ptr::null(), ptr::null_mut())
        };

        let status = unsafe {
            sgx_rijndael128GCM_decrypt(
                &self.key.key as *const Key128bit,
                p_src,
                src_len as u32,
                p_dst,
                self.iv.as_ref().as_ptr(),
                self.iv.as_ref().len() as u32,
                p_aad,
                aad_len as u32,
                mac as *const Mac128bit,
            )
        };

        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn decrypt_in_place(&mut self, in_out: &mut [u8], mac: &Mac128bit) -> SgxResult {
        let mut dst = vec![0_u8; in_out.len()];
        self.decrypt(in_out, dst.as_mut_slice(), mac)?;
        in_out.clone_from_slice(dst.as_slice());
        Ok(())
    }

    pub fn mac(&mut self) -> SgxResult<Mac128bit> {
        let aad = self.aad.as_ref();
        ensure!(!aad.is_empty(), SgxStatus::InvalidParameter);

        let mut mac = AlignMac128bit::default();
        let status = unsafe {
            sgx_rijndael128GCM_encrypt(
                &self.key.key as *const Key128bit,
                ptr::null(),
                0,
                ptr::null_mut(),
                self.iv.as_ref().as_ptr(),
                self.iv.as_ref().len() as u32,
                aad.as_ptr(),
                aad.len() as u32,
                &mut mac.mac as *mut Mac128bit,
            )
        };

        ensure!(status.is_success(), status);
        Ok(mac.mac)
    }

    pub fn verify_mac(&mut self, mac: &Mac128bit) -> SgxResult {
        let aad = self.aad.as_ref();
        ensure!(!aad.is_empty(), SgxStatus::InvalidParameter);

        let status = unsafe {
            sgx_rijndael128GCM_decrypt(
                &self.key.key as *const Key128bit,
                ptr::null(),
                0,
                ptr::null_mut(),
                self.iv.as_ref().as_ptr(),
                self.iv.as_ref().len() as u32,
                aad.as_ptr(),
                aad.len() as u32,
                mac as *const Mac128bit,
            )
        };

        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn enc_update(&mut self, src: &[u8], dst: &mut [u8]) -> SgxResult {
        let src_len = src.len();
        let dst_len = dst.len();

        ensure!(
            (1..i32::MAX as usize).contains(&src_len),
            SgxStatus::InvalidParameter
        );
        ensure!(src_len == dst_len, SgxStatus::InvalidParameter);

        if self.handle.is_null() {
            self.init()?;
        }

        let status = unsafe {
            sgx_aes_gcm128_enc_update(src.as_ptr(), src_len as u32, dst.as_mut_ptr(), self.handle)
        };

        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn enc_update_in_place(&mut self, in_out: &mut [u8]) -> SgxResult {
        let mut dst = vec![0_u8; in_out.len()];
        self.enc_update(in_out, dst.as_mut_slice())?;
        in_out.clone_from_slice(dst.as_slice());
        Ok(())
    }

    pub unsafe fn dec_update(&mut self, src: &[u8], dst: &mut [u8]) -> SgxResult {
        let src_len = src.len();
        let dst_len = dst.len();

        ensure!(
            (1..i32::MAX as usize).contains(&src_len),
            SgxStatus::InvalidParameter
        );
        ensure!(src_len == dst_len, SgxStatus::InvalidParameter);

        if self.handle.is_null() {
            self.init()?;
        }

        let status =
            sgx_aes_gcm128_dec_update(src.as_ptr(), src_len as u32, dst.as_mut_ptr(), self.handle);

        ensure!(status.is_success(), status);
        Ok(())
    }

    pub unsafe fn dec_update_in_place(&mut self, in_out: &mut [u8]) -> SgxResult {
        let mut dst = vec![0_u8; in_out.len()];
        self.dec_update(in_out, dst.as_mut_slice())?;
        in_out.clone_from_slice(dst.as_slice());
        Ok(())
    }

    pub fn enc_get_mac(&mut self) -> SgxResult<Mac128bit> {
        let mut mac = AlignMac128bit::default();
        let status = unsafe { sgx_aes_gcm128_enc_get_mac(&mut mac.mac as *mut u8, self.handle) };

        ensure!(status.is_success(), status);
        Ok(mac.mac)
    }

    pub fn dec_verify_mac(&mut self, mac: &Mac128bit) -> SgxResult {
        let status = unsafe { sgx_aes_gcm128_dec_verify_mac(mac as *const u8, self.handle) };

        ensure!(status.is_success(), status);
        Ok(())
    }

    fn init(&mut self) -> SgxResult {
        let aad = self.aad.as_ref();
        let aad_len = aad.len();

        ensure!(aad_len < i32::MAX as usize, SgxStatus::InvalidParameter);

        let p_aad = if !aad.is_empty() {
            aad.as_ptr()
        } else {
            ptr::null()
        };

        let status = unsafe {
            sgx_aes_gcm128_init(
                &self.key.key as *const u8,
                self.iv.as_ref().as_ptr(),
                self.iv.as_ref().len() as u32,
                p_aad,
                aad_len as u32,
                &mut self.handle,
            )
        };

        ensure!(status.is_success(), status);
        Ok(())
    }
}

impl Default for AesGcm<[u8; 0]> {
    fn default() -> AesGcm<[u8; 0]> {
        let mut key = AlignKey128bit::default();
        super::rand(&mut key.key);

        AesGcm {
            key,
            iv: Nonce::new(),
            aad: Aad::default(),
            handle: ptr::null_mut(),
        }
    }
}

impl<A: AsRef<[u8]>> Drop for AesGcm<A> {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { sgx_aes_gcm_close(self.handle) };
        }
    }
}

pub struct Aad<A>(A);

impl<A: AsRef<[u8]>> Aad<A> {
    #[inline]
    pub fn from(aad: A) -> Aad<A> {
        Aad(aad)
    }
}

impl<A> AsRef<[u8]> for Aad<A>
where
    A: AsRef<[u8]>,
{
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Aad<[u8; 0]> {
    pub fn empty() -> Aad<[u8; 0]> {
        Self::from([])
    }
}

impl Default for Aad<[u8; 0]> {
    fn default() -> Aad<[u8; 0]> {
        Aad::empty()
    }
}

impl<A> Clone for Aad<A>
where
    A: Clone,
{
    #[inline]
    fn clone(&self) -> Aad<A> {
        Self(self.0.clone())
    }
}

impl<A> Copy for Aad<A> where A: Copy {}

impl<A> core::fmt::Debug for Aad<A>
where
    A: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Aad").field(&self.0).finish()
    }
}

impl<A> PartialEq for Aad<A>
where
    A: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<A> Eq for Aad<A> where A: Eq {}

impl<A> ConstTimeEq for Aad<A>
where
    A: ConstTimeEq + BytewiseEquality + Eq,
{
    fn ct_eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0)
    }
}

unsafe impl<A: ContiguousMemory> ContiguousMemory for Aad<A> {}

unsafe impl<A: BytewiseEquality> BytewiseEquality for Aad<A> {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Nonce([u8; AESGCM_IV_SIZE]);

impl Nonce {
    pub fn new() -> Nonce {
        let mut nonce = [0_u8; AESGCM_IV_SIZE];
        super::rand(&mut nonce);
        Nonce(nonce)
    }

    #[inline]
    pub fn zeroed() -> Nonce {
        Nonce([0_u8; AESGCM_IV_SIZE])
    }
}

impl Default for Nonce {
    #[inline]
    fn default() -> Nonce {
        Nonce::new()
    }
}

impl AsRef<[u8; AESGCM_IV_SIZE]> for Nonce {
    #[inline]
    fn as_ref(&self) -> &[u8; AESGCM_IV_SIZE] {
        &self.0
    }
}

impl From<[u8; AESGCM_IV_SIZE]> for Nonce {
    #[inline]
    fn from(nonce: [u8; AESGCM_IV_SIZE]) -> Nonce {
        Nonce(nonce)
    }
}

impl From<&[u8; AESGCM_IV_SIZE]> for Nonce {
    #[inline]
    fn from(nonce: &[u8; AESGCM_IV_SIZE]) -> Nonce {
        Nonce(*nonce)
    }
}

impl TryFrom<&[u8]> for Nonce {
    type Error = TryFromSliceError;

    fn try_from(nonce: &[u8]) -> Result<Nonce, TryFromSliceError> {
        let nonce: &[u8; AESGCM_IV_SIZE] = nonce.try_into()?;
        Ok(Nonce(*nonce))
    }
}

unsafe impl ContiguousMemory for Nonce {}

unsafe impl BytewiseEquality for Nonce {}
