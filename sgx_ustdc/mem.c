// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

#include <sys/types.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <errno.h>

void *u_malloc_ocall(int * error, size_t size)
{
    void * ret = malloc(size);
    if (error) {
        *error = ret == NULL ? errno : 0;
    }
}

void u_free_ocall(void *p)
{
    free(p);
}

void *u_mmap_ocall(int * error, void * start, size_t length, int prot, int flags, int fd, off_t offset)
{
    void * ret = mmap(start, length, prot, flags, fd, offset);
    if (error) {
        *error = ret == MAP_FAILED ? errno : 0;
    }
    return ret;
}

int u_munmap_ocall(int * error, void * start, size_t length)
{
    int ret = munmap(start, length);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_msync_ocall(int * error, void * addr, size_t length, int flags)
{
    int ret = msync(addr, length, flags);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_mprotect_ocall(int * error, void * addr, size_t length, int prot)
{
    int ret = mprotect(addr, length, prot);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}