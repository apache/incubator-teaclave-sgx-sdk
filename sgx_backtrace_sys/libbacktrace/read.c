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

/* This file implements file views when mmap is not available.  */

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

    host = sgx_ocalloc(size);
    if (host == NULL) {
        error_callback(data, "sgx ocalloc failed", ENOMEM);
        return 0;
    }

    view->base = backtrace_alloc(state, size, error_callback, data);
    if (view->base == NULL) {
        sgx_ocfree();
        return 0;
    }

    view->data = view->base;
    view->len = size;

    status = u_read_ocall((size_t *)&got, &error, descriptor, host, size);
    if (status != 0) {
        error_callback(data, "sgx ocall failed", status);
        free(view->base);
        sgx_ocfree();
        return 0;
    }
    if (got < 0) {
        error_callback(data, "read", error);
        free(view->base);
        return 0;
    }

    if ((size_t) got < size) {
        error_callback(data, "file too short", 0);
        free(view->base);
        sgx_ocfree();
        return 0;
    }

    memcpy(view->base, host, size);
    sgx_ocfree();

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
