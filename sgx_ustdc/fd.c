// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#define _LARGEFILE64_SOURCE

#include <sys/types.h>
#include <sys/ioctl.h>
#include <errno.h>
#include <unistd.h>
#include <fcntl.h>

ssize_t u_read_ocall(int * error, int fd, void * buf, size_t count)
{
    ssize_t ret = read(fd, buf, count);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_pread64_ocall(int * error, int fd, void * buf, size_t count, off64_t offset)
{
    ssize_t ret = pread64(fd, buf, count, offset);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_write_ocall(int * error, int fd, const void * buf, size_t count)
{
    ssize_t ret = write(fd, buf, count);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_pwrite64_ocall(int * error, int fd, const void * buf, size_t count, off64_t offset)
{
    ssize_t ret = pwrite64(fd, buf, count, offset);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fcntl_arg0_ocall(int * error, int fd, int cmd)
{
    int ret = fcntl(fd, cmd);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fcntl_arg1_ocall(int * error, int fd, int cmd, int arg)
{
    int ret = fcntl(fd, cmd, arg);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_ioctl_arg0_ocall(int * error, int fd, int request)
{
    int ret = ioctl(fd, request);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_ioctl_arg1_ocall(int * error, int fd, int request, int * arg)
{
    int ret = ioctl(fd, request, arg);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_close_ocall(int * error, int fd)
{
    int ret = close(fd);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}