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

use crate::os::fd::owned::OwnedFd;
use crate::os::fd::raw::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use crate::sys_common::{self, AsInner, FromInner, IntoInner};
use crate::{net, sys};

macro_rules! impl_as_raw_fd {
    ($($t:ident)*) => {$(
        impl AsRawFd for net::$t {
            #[inline]
            fn as_raw_fd(&self) -> RawFd {
                self.as_inner().socket().as_raw_fd()
            }
        }
    )*};
}
impl_as_raw_fd! { TcpStream TcpListener UdpSocket }

macro_rules! impl_from_raw_fd {
    ($($t:ident)*) => {$(
        impl FromRawFd for net::$t {
            #[inline]
            unsafe fn from_raw_fd(fd: RawFd) -> net::$t {
                unsafe {
                    let socket = sys::net::Socket::from_inner(FromInner::from_inner(OwnedFd::from_raw_fd(fd)));
                    net::$t::from_inner(sys_common::net::$t::from_inner(socket))
                }
            }
        }
    )*};
}
impl_from_raw_fd! { TcpStream TcpListener UdpSocket }

macro_rules! impl_into_raw_fd {
    ($($t:ident)*) => {$(
        impl IntoRawFd for net::$t {
            #[inline]
            fn into_raw_fd(self) -> RawFd {
                self.into_inner().into_socket().into_inner().into_inner().into_raw_fd()
            }
        }
    )*};
}
impl_into_raw_fd! { TcpStream TcpListener UdpSocket }
