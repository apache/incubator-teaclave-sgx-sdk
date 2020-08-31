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

#include <sys/types.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <errno.h>

void *u_malloc_ocall(int *error, size_t size)
{
    void *ret = malloc(size);
    if (error) {
        *error = ret == NULL ? errno : 0;
    }
    return ret;
}

void u_free_ocall(void *p)
{
    free(p);
}

void *u_mmap_ocall(int *error, void *start, size_t length, int prot, int flags, int fd, off_t offset)
{
    void *ret = mmap(start, length, prot, flags, fd, offset);
    if (error) {
        *error = ret == MAP_FAILED ? errno : 0;
    }
    return ret;
}

int u_munmap_ocall(int *error, void *start, size_t length)
{
    int ret = munmap(start, length);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_msync_ocall(int *error, void *addr, size_t length, int flags)
{
    int ret = msync(addr, length, flags);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_mprotect_ocall(int *error, void *addr, size_t length, int prot)
{
    int ret = mprotect(addr, length, prot);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}