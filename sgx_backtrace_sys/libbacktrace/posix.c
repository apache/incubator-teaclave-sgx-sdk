/* posix.c -- POSIX file I/O routines for the backtrace library.
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
#include <sys/types.h>
#include <sys/stat.h>
//#include <fcntl.h>
//#include <unistd.h>

#include "backtrace.h"
#include "backtrace_t.h"
#include "internal.h"

#ifndef O_RDONLY
#define O_RDONLY 0
#endif

#ifndef O_WRONLY
#define O_WRONLY 1
#endif

#ifndef O_BINARY
#define O_BINARY 0
#endif

#ifndef O_CLOEXEC
#define O_CLOEXEC 02000000
#endif

#ifndef F_SETFD
#define F_SETFD 2
#endif

#ifndef FD_CLOEXEC
#define FD_CLOEXEC 1
#endif

/* Open a file for reading.  */

int
backtrace_open(const char* filename, backtrace_error_callback error_callback,
               void* data, int* does_not_exist) {
    int descriptor = 0;
    int error = 0;

    if (does_not_exist != NULL) {
        *does_not_exist = 0;
    }

    uint32_t status = u_open_ocall(&descriptor, &error, filename,
                      (int)(O_RDONLY | O_BINARY | O_CLOEXEC));

    if (status != 0) {
        error_callback(data, "sgx ocall failed", status);
        return -1;
    }

    //descriptor = open (filename, (int) (O_RDONLY | O_BINARY | O_CLOEXEC));
    if (descriptor < 0) {
        if (does_not_exist != NULL && error == ENOENT) {
            *does_not_exist = 1;
        } else {
            error_callback(data, filename, error);
        }

        return -1;
    }

#ifdef HAVE_FCNTL
    /* Set FD_CLOEXEC just in case the kernel does not support
       O_CLOEXEC. It doesn't matter if this fails for some reason.
       FIXME: At some point it should be safe to only do this if
       O_CLOEXEC == 0.  */
    //fcntl (descriptor, F_SETFD, FD_CLOEXEC);
    int retval = 0;
    u_fcntl_arg1_ocall(&retval, &error, descriptor, F_SETFD, FD_CLOEXEC);
#endif

    return descriptor;
}

/* Close DESCRIPTOR.  */

int
backtrace_close(int descriptor, backtrace_error_callback error_callback,
                void* data) {

    int retval = 0;
    int error = 0;
    uint32_t status = u_close_ocall(&retval, &error, descriptor);

    if (status != 0) {
        error_callback(data, "sgx ocall failed", status);
        return 0;
    }

    if (retval < 0) {
        error_callback(data, "close", error);
        return 0;
    }

    /*
    if (close (descriptor) < 0)
    {
      error_callback (data, "close", errno);
      return 0;
    }
    */
    return 1;
}
