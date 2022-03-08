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

use super::*;
use core::cmp;
use core::mem;
use core::num::NonZeroUsize;
use core::slice;
use sgx_trts::trts::is_within_enclave;

impl_bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct MsgHdrFlags: i32 {
        const MSG_OOB          = 0x01;
        const MSG_CTRUNC       = 0x08;
        const MSG_TRUNC        = 0x20;
        const MSG_EOR          = 0x80;
        const MSG_ERRQUEUE     = 0x2000;
    }
}

#[derive(Debug)]
pub struct MsgHdr<'a> {
    pub name: Option<&'a [u8]>,
    pub iovs: Vec<&'a [u8]>,
    pub control: Option<&'a [u8]>,
    pub flags: MsgHdrFlags,
}

#[derive(Debug)]
pub struct MsgHdrMut<'a> {
    pub name: Option<&'a mut [u8]>,
    pub name_len: u32,
    pub iovs: Vec<&'a mut [u8]>,
    pub control: Option<&'a mut [u8]>,
    pub control_len: usize,
    pub flags: MsgHdrFlags,
}

impl<'a> MsgHdr<'a> {
    pub unsafe fn from_raw(c_msg: &msghdr) -> Self {
        let name = new_optional_slice(c_msg.msg_name as *const u8, c_msg.msg_namelen as usize);
        let iovs = new_optional_slice(c_msg.msg_iov as *const iovec, c_msg.msg_iovlen as usize);
        let control = new_optional_slice(
            c_msg.msg_control as *const u8,
            c_msg.msg_controllen as usize,
        );
        let flags = MsgHdrFlags::from_bits_truncate(c_msg.msg_flags);
        let iovs = iovs
            .map(|iovs_slice| {
                iovs_slice
                    .iter()
                    .flat_map(|iov| new_optional_slice(iov.iov_base as *const u8, iov.iov_len))
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        Self {
            name,
            iovs,
            control,
            flags,
        }
    }
}

impl<'a> MsgHdrMut<'a> {
    pub unsafe fn from_raw(c_msg: &mut msghdr) -> Self {
        let name = new_optional_slice_mut(c_msg.msg_name as *mut u8, c_msg.msg_namelen as usize);
        let iovs = new_optional_slice_mut(c_msg.msg_iov as *mut iovec, c_msg.msg_iovlen as usize);
        let control =
            new_optional_slice_mut(c_msg.msg_control as *mut u8, c_msg.msg_controllen as usize);
        let flags = MsgHdrFlags::from_bits_truncate(c_msg.msg_flags);
        let iovs = iovs
            .map(|iovs_slice| {
                iovs_slice
                    .iter()
                    .flat_map(|iov| new_optional_slice_mut(iov.iov_base as *mut u8, iov.iov_len))
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let name_len = name.as_ref().map(|n| n.len()).unwrap_or(0) as u32;
        let control_len = control.as_ref().map(|c| c.len()).unwrap_or(0);
        Self {
            name,
            name_len,
            iovs,
            control,
            control_len,
            flags,
        }
    }
}

unsafe fn new_optional_slice<'a, T>(ptr: *const T, size: usize) -> Option<&'a [T]> {
    if !ptr.is_null() && size > 0 {
        let slice = slice::from_raw_parts::<T>(ptr, size);
        Some(slice)
    } else {
        None
    }
}

unsafe fn new_optional_slice_mut<'a, T>(ptr: *mut T, size: usize) -> Option<&'a mut [T]> {
    if !ptr.is_null() && size > 0 {
        let slice = slice::from_raw_parts_mut::<T>(ptr, size);
        Some(slice)
    } else {
        None
    }
}

trait AsLibcIovec {
    fn as_libc_iovec(&self) -> iovec;
}

impl AsLibcIovec for &[u8] {
    fn as_libc_iovec(&self) -> iovec {
        let (iov_base, iov_len) = self.as_ptr_and_len();
        let iov_base = iov_base as *mut c_void;
        iovec { iov_base, iov_len }
    }
}

impl AsLibcIovec for &mut [u8] {
    fn as_libc_iovec(&self) -> iovec {
        let (iov_base, iov_len) = self.as_ptr_and_len();
        let iov_base = iov_base as *mut c_void;
        iovec { iov_base, iov_len }
    }
}

impl AsLibcIovec for HostSlice<'_> {
    fn as_libc_iovec(&self) -> iovec {
        self.as_slice().as_libc_iovec()
    }
}

impl AsLibcIovec for HostSliceMut<'_> {
    fn as_libc_iovec(&self) -> iovec {
        self.as_slice().as_libc_iovec()
    }
}

pub unsafe fn sendmsg(sockfd: c_int, msg: &MsgHdr, flags: c_int) -> OCallResult<size_t> {
    ensure!(
        is_within_enclave(msg as *const _ as *const u8, mem::size_of_val(msg)),
        ecust!("Buffer is not strictly inside enclave")
    );
    ensure!(
        is_within_enclave(
            msg.iovs.as_ptr() as *const u8,
            msg.iovs.capacity() * mem::size_of::<&[u8]>()
        ),
        ecust!("Buffer is not strictly inside enclave")
    );

    let data_len = NonZeroUsize::new(msg.iovs.iter().map(|s| s.len()).sum())
        .ok_or_else(|| OCallError::from_custom_error("Null buffer"))?;

    let (msg_name, msg_namelen) = msg.name.as_ptr_and_len();
    let (msg_control, msg_controllen) = msg.control.as_ptr_and_len();

    let heap_buf = HeapBuffer::alloc(data_len)?;
    let mut host_buf = HostBuffer::from_heap_buffer(heap_buf);
    let mut host_pos = 0_usize;

    let io_data: Vec<iovec> = msg
        .iovs
        .iter()
        .filter(|&&slice| !slice.is_empty())
        .flat_map(|&slice| {
            let mut host_slice = host_buf.range_mut(host_pos..host_pos + slice.len());
            host_slice
                .copy_from_enclave(slice)
                .map(|_| {
                    host_pos += slice.len();
                    host_slice.as_libc_iovec()
                })
                .ok()
        })
        .collect();
    let (msg_iov, msg_iovlen) = io_data.as_slice().as_ptr_and_len();

    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let status = u_sendmsg_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        sockfd,
        msg_name as *const c_void,
        msg_namelen as socklen_t,
        msg_iov,
        msg_iovlen,
        msg_control as *const c_void,
        msg_controllen,
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nsent = result as usize;
    ensure!(nsent <= data_len.get(), ecust!("Malformed return size"));

    Ok(nsent)
}

pub unsafe fn recvmsg(sockfd: c_int, msg: &mut MsgHdrMut, flags: c_int) -> OCallResult<size_t> {
    ensure!(
        is_within_enclave(msg as *const _ as *const u8, mem::size_of_val(msg)),
        ecust!("Buffer is not strictly inside enclave")
    );
    ensure!(
        is_within_enclave(
            msg.iovs.as_ptr() as *const u8,
            msg.iovs.capacity() * mem::size_of::<&[u8]>()
        ),
        ecust!("Buffer is not strictly inside enclave")
    );

    let mut total_size: usize = 0;
    for s in msg.iovs.iter() {
        check_trusted_enclave_buffer(*s)?;
        total_size = total_size
            .checked_add(s.len())
            .ok_or_else(|| OCallError::from_custom_error("Overflow"))?;
    }

    let data_len = NonZeroUsize::new(total_size)
        .ok_or_else(|| OCallError::from_custom_error("Null buffer"))?;

    let (msg_name, msg_namelen) = msg.name.as_mut_ptr_and_len();
    let (msg_control, msg_controllen) = msg.control.as_mut_ptr_and_len();
    ensure!(msg_namelen as u32 == msg.name_len, eos!(EINVAL));
    ensure!(msg_controllen == msg.control_len, eos!(EINVAL));

    let mut msg_namelen_out = 0_u32;
    let mut msg_controllen_out = 0_usize;
    let mut msg_flags = 0;

    let heap_buf = HeapBuffer::alloc_zeroed(data_len)?;
    let mut host_buf = HostBuffer::from_heap_buffer(heap_buf);
    let mut host_pos = 0_usize;

    let mut io_data: Vec<iovec> = msg
        .iovs
        .iter()
        .filter(|&slice| !slice.is_empty())
        .map(|slice| {
            let host_slice = host_buf.range_mut(host_pos..host_pos + slice.len());
            host_pos += slice.len();
            host_slice.as_libc_iovec()
        })
        .collect();
    let (msg_iov, msg_iovlen) = io_data.as_mut_slice().as_mut_ptr_and_len();

    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let status = u_recvmsg_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        sockfd,
        msg_name as *mut c_void,
        msg_namelen as socklen_t,
        &mut msg_namelen_out as *mut socklen_t,
        msg_iov,
        msg_iovlen,
        msg_control as *mut c_void,
        msg_controllen,
        &mut msg_controllen_out as *mut usize,
        &mut msg_flags as *mut i32,
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    ensure!(
        msg_namelen_out <= msg_namelen as socklen_t,
        ecust!("Malformed return size")
    );
    ensure!(
        msg_controllen_out <= msg_controllen,
        ecust!("Malformed return size")
    );

    let msg_flags = MsgHdrFlags::from_bits_truncate(msg_flags);

    let nrecv = result as usize;
    // For MSG_TRUNC recvmsg returns the real length of the packet or datagram,
    // even when it was longer than the passed buffer.
    if flags & MSG_TRUNC > 0 && nrecv > data_len.get() {
        ensure!(
            msg_flags.contains(MsgHdrFlags::MSG_TRUNC),
            ecust!("Malformed return size")
        );
    } else {
        ensure!(nrecv <= data_len.get(), ecust!("Malformed return size"));
    }

    let copy_len = cmp::min(nrecv, data_len.get());
    let mut remain_len = copy_len;
    for buf in msg.iovs.iter_mut() {
        let pos = copy_len - remain_len;
        let n = cmp::min(buf.len(), remain_len);
        host_buf
            .as_slice()
            .range(pos..pos + n)
            .copy_to_enclave(&mut (*buf)[..n])?;

        remain_len -= n;
        if remain_len == 0 {
            break;
        }
    }

    msg.name_len = msg_namelen_out as u32;
    msg.control_len = msg_controllen_out;
    msg.flags = msg_flags;

    Ok(nrecv)
}
