/* read.c -- File views without mmap.
   Copyright (C) 2012-2018 Free Software Foundation, Inc.
   Written by Ian Lance Taylor, Google.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

    (1) Redistributions of source code must retain the above copyright
    notice, this list of conditions and the following disclaimer.

    (2) Redistributions in binary form must reproduce the above copyright
    notice, this list of conditions and the following disclaimer in
    the documentation and/or other materials provided with the
    distribution.

    (3) The name of the author may not be used to
    endorse or promote products derived from this software without
    specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY EXPRESS OR
IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY DIRECT,
INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING
IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
POSSIBILITY OF SUCH DAMAGE.  */

#include "config.h"

#include <errno.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

#include "backtrace.h"
#include "backtrace_t.h"
#include "internal.h"

#include "sgx_edger8r.h"
#include "sgx_trts.h"

/* This file implements file views when mmap is not available.  */

static
size_t shared_buf_remain_size(void) {
    size_t ocall_size = 8 * sizeof(size_t);
    size_t remain_size = sgx_ocremain_size();
    if (remain_size <= ocall_size) {
        return 0;
    }

    return remain_size - ocall_size;
}

static
ssize_t read_host_buf(const void* host, void* encl_buf, size_t size) {
    size_t left_size = size;
    size_t offset = 0;
    size_t nread = 0;
    int error = 0;
    uint32_t status = 0;
    size_t copy_size = 0;
    void *src = (void *)host;
    void *dest = encl_buf;

    size_t remain_size = shared_buf_remain_size();
    if (remain_size == 0) {
        return -1;
    }

    while (left_size > 0) {
        copy_size = (left_size > remain_size) ? remain_size : left_size;

        nread = 0;
        status = u_read_hostbuf_ocall(&nread, &error, src, dest, copy_size);
        if ((status != 0) || (nread != copy_size)) {
            return (ssize_t)(size - left_size);
        }

        src = (void *)((size_t)src + copy_size);
        dest = (void *)((size_t)dest + copy_size);
        left_size -= copy_size;
    }

    return (ssize_t)size;
}

/* Create a view of SIZE bytes from DESCRIPTOR at OFFSET.  */

int
backtrace_get_view(struct backtrace_state* state, int descriptor,
                   off_t offset, size_t size,
                   backtrace_error_callback error_callback,
                   void* data, struct backtrace_view* view) {
    ssize_t got = 0;
    int error = 0;
    off_t retval = 0;
    uint32_t status = 0;
    void *host = NULL;

    status = u_lseek_ocall((uint64_t *)&retval, &error, descriptor, offset, SEEK_SET);
    if (status != 0) {
        error_callback(data, "sgx ocall failed", status);
        return 0;
    }
    if (retval < 0) {
        error_callback(data, "lseek", error);
        return 0;
    }

    status = u_malloc_ocall(&host, &error, size, 1, 1);
    if (status != 0) {
        error_callback(data, "sgx ocall failed", status);
        return 0;
    }

    if (host == NULL) {
        error_callback(data, "malloc_ocall failed", ENOMEM);
        return 0;
    }

    if (!sgx_is_outside_enclave(host, size)) {
        error_callback(data, "malloc_ocall failed", error);
        return 0;
    }

    view->base = backtrace_alloc(state, size, error_callback, data);
    if (view->base == NULL) {
        u_free_ocall(host);
        return 0;
    }

    view->data = view->base;
    view->len = size;

    status = u_read_ocall((size_t *)&got, &error, descriptor, host, size);
    if (status != 0) {
        error_callback(data, "sgx ocall failed", status);
        free(view->base);
        u_free_ocall(host);
        return 0;
    }
    if (got < 0) {
        error_callback(data, "read", error);
        free(view->base);
        u_free_ocall(host);
        return 0;
    }

    if ((size_t) got < size) {
        error_callback(data, "file too short", 0);
        free(view->base);
        u_free_ocall(host);
        return 0;
    }

    ssize_t nread = read_host_buf(host, view->base, size);
    if (nread < 0) {
        error_callback(data, "read hostbuf failed", ENOMEM);
        free(view->base);
        u_free_ocall(host);
    }
    if ((size_t)nread != size) {
        error_callback(data, "read hostbuf failed", 0);
        free(view->base);
        u_free_ocall(host);
        return 0;
    }

    u_free_ocall(host);

    return 1;
}

/* Release a view read by backtrace_get_view.  */

void
backtrace_release_view(struct backtrace_state* state,
                       struct backtrace_view* view,
                       backtrace_error_callback error_callback,
                       void* data) {
    backtrace_free(state, view->base, view->len, error_callback, data);
    view->data = NULL;
    view->base = NULL;
}
