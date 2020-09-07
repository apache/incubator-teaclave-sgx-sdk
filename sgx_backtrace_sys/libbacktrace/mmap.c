/* mmap.c -- Memory allocation with mmap.
   Copyright (C) 2012-2016 Free Software Foundation, Inc.
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
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>

#include "backtrace.h"
#include "backtrace_t.h"
#include "internal.h"

#include "sgx_trts.h"
/* Memory allocation on systems that provide anonymous mmap.  This
   permits the backtrace functions to be invoked from a signal
   handler, assuming that mmap is async-signal safe.  */

#ifndef PROT_READ
#define PROT_READ 0x1
#endif

#ifndef PROT_WRITE
#define PROT_WRITE 0x2
#endif

#ifndef MAP_SHARED
#define MAP_SHARED 0x01
#endif

#ifndef MAP_PRIVATE
#define MAP_PRIVATE 0x02
#endif

#ifndef MAP_ANONYMOUS
#define MAP_ANONYMOUS 0x20
#endif

#ifndef MAP_FAILED
#define MAP_FAILED ((void *)-1)
#endif

extern int getpagesize(void);

/* A list of free memory blocks.  */

struct backtrace_freelist_struct {
    /* Next on list.  */
    struct backtrace_freelist_struct* next;
    /* Size of this block, including this structure.  */
    size_t size;
};

/* Free memory allocated by backtrace_alloc.  */

static void
backtrace_free_locked (struct backtrace_state *state, void *addr, size_t size) {
  /* Just leak small blocks.  We don't have to be perfect.  Don't put
     more than 16 entries on the free list, to avoid wasting time
     searching when allocating a block.  If we have more than 16
     entries, leak the smallest entry.  */

    if (size >= sizeof (struct backtrace_freelist_struct)) {
        size_t c = 0;
        struct backtrace_freelist_struct **ppsmall = NULL;
        struct backtrace_freelist_struct **pp = NULL;
        struct backtrace_freelist_struct *p = NULL;

        for (pp = &state->freelist; *pp != NULL; pp = &(*pp)->next) {
	        if (ppsmall == NULL || (*pp)->size < (*ppsmall)->size) {
                ppsmall = pp;
            }
	        ++c;
	    }
        if (c >= 16) {
	        if (size <= (*ppsmall)->size) {
                return;
            }
	        *ppsmall = (*ppsmall)->next;
        }

        p = (struct backtrace_freelist_struct *) addr;
        p->next = state->freelist;
        p->size = size;
        state->freelist = p;
    }
}

/* Allocate memory like malloc.  If ERROR_CALLBACK is NULL, don't
   report an error.  */

void*
backtrace_alloc(struct backtrace_state* state,
                size_t size, backtrace_error_callback error_callback,
                void* data) {
    void* ret = NULL;
    int locked = 0;
    struct backtrace_freelist_struct** pp = NULL;
    size_t pagesize = 0;
    size_t asksize = 0;
    void* page = NULL;
    int error = 0;

    ret = NULL;

    /* If we can acquire the lock, then see if there is space on the
       free list.  If we can't acquire the lock, drop straight into
       using mmap.  __sync_lock_test_and_set returns the old state of
       the lock, so we have acquired it if it returns 0.  */

    if (!state->threaded) {
        locked = 1;
    } else {
        locked = __sync_lock_test_and_set(&state->lock_alloc, 1) == 0;
    }

    if (locked) {
        for (pp = &state->freelist; *pp != NULL; pp = &(*pp)->next) {
            if ((*pp)->size >= size) {
                struct backtrace_freelist_struct* p;

                p = *pp;
                *pp = p->next;

                /* Round for alignment; we assume that no type we care about
                     is more than 8 bytes.  */
                size = (size + 7) & ~(size_t) 7;

                if (size < p->size) {
                    backtrace_free_locked(state, (char*) p + size, p->size - size);
                }

                ret = (void*) p;
                break;
            }
        }

        if (state->threaded) {
            __sync_lock_release(&state->lock_alloc);
        }
    }

    if (ret == NULL) {
        /* Allocate a new page.  */

        pagesize = getpagesize();
        asksize = (size + pagesize - 1) & ~(pagesize - 1);

        uint32_t status = u_mmap_ocall(&page, &error, NULL, asksize, PROT_READ | PROT_WRITE,
                          MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);

        if (status != 0) {
            error_callback(data, "sgx ocall failed", status);
            return ret;
        }

        if (!sgx_is_outside_enclave(page, asksize)) {
            error_callback(data, "mmap result error", error);
            return 0;
        }

        //page = mmap (NULL, asksize, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        if (page == MAP_FAILED) {
            if (error_callback) {
                error_callback(data, "mmap", error);
            }
        } else {
            size = (size + 7) & ~(size_t) 7;

            if (size < asksize)
                backtrace_free(state, (char*) page + size, asksize - size,
                               error_callback, data);

            ret = page;
        }
    }

    return ret;
}

/* Free memory allocated by backtrace_alloc.  */

void
backtrace_free(struct backtrace_state* state, void* addr, size_t size,
               backtrace_error_callback error_callback ATTRIBUTE_UNUSED,
               void* data ATTRIBUTE_UNUSED) {
    int locked = 0;

    /* If we are freeing a large aligned block, just release it back to
       the system.  This case arises when growing a vector for a large
       binary with lots of debug info.  Calling munmap here may cause us
       to call mmap again if there is also a large shared library; we
       just live with that.  */
    if (size >= 16 * 4096) {
        size_t pagesize = 0;

        pagesize = getpagesize();

        if (((uintptr_t) addr & (pagesize - 1)) == 0
                && (size & (pagesize - 1)) == 0) {
            /* If munmap fails for some reason, just add the block to
             the freelist.  */
            //if (munmap (addr, size) == 0)
            //return;
            int retval = 0;
            int error = 0;
            uint32_t status = u_munmap_ocall(&retval, &error, addr, size);

            if (status == 0 && retval == 0) {
                return;
            }

            if (status != 0) {
                error_callback(data, "sgx ocall failed", status);
            } else if (retval == -1) {
                error_callback(data, "munmap", error);
            }
        }
    }

    /* If we can acquire the lock, add the new space to the free list.
       If we can't acquire the lock, just leak the memory.
       __sync_lock_test_and_set returns the old state of the lock, so we
       have acquired it if it returns 0.  */

    if (!state->threaded) {
        locked = 1;
    } else {
        locked = __sync_lock_test_and_set(&state->lock_alloc, 1) == 0;
    }

    if (locked) {
        backtrace_free_locked(state, addr, size);

        if (state->threaded) {
            __sync_lock_release(&state->lock_alloc);
        }
    }
}

/* Grow VEC by SIZE bytes.  */

void*
backtrace_vector_grow(struct backtrace_state* state, size_t size,
                      backtrace_error_callback error_callback,
                      void* data, struct backtrace_vector* vec) {
    void* ret = NULL;

    if (size > vec->alc) {
        size_t pagesize = 0;
        size_t alc = 0;
        void* base = NULL;

        pagesize = getpagesize();
        alc = vec->size + size;

        if (vec->size == 0) {
            alc = 16 * size;
        } else if (alc < pagesize) {
            alc *= 2;

            if (alc > pagesize) {
                alc = pagesize;
            }
        } else {
            alc *= 2;
            alc = (alc + pagesize - 1) & ~(pagesize - 1);
        }

        base = backtrace_alloc(state, alc, error_callback, data);

        if (base == NULL) {
            return NULL;
        }

        if (vec->base != NULL) {
            memcpy(base, vec->base, vec->size);
            backtrace_free(state, vec->base, vec->size + vec->alc,
                           error_callback, data);
        }

        vec->base = base;
        vec->alc = alc - vec->size;
    }

    ret = (char*) vec->base + vec->size;
    vec->size += size;
    vec->alc -= size;
    return ret;
}

/* Finish the current allocation on VEC.  */

void*
backtrace_vector_finish(
    struct backtrace_state* state ATTRIBUTE_UNUSED,
    struct backtrace_vector* vec,
    backtrace_error_callback error_callback ATTRIBUTE_UNUSED,
    void* data ATTRIBUTE_UNUSED) {
    void* ret = NULL;

    ret = vec->base;
    vec->base = (char*) vec->base + vec->size;
    vec->size = 0;
    return ret;
}

/* Release any extra space allocated for VEC.  */

int
backtrace_vector_release(struct backtrace_state* state,
                         struct backtrace_vector* vec,
                         backtrace_error_callback error_callback,
                         void* data) {
    size_t size = 0;
    size_t alc = 0;
    size_t aligned = 0;

    /* Make sure that the block that we free is aligned on an 8-byte
       boundary.  */
    size = vec->size;
    alc = vec->alc;
    aligned = (size + 7) & ~(size_t) 7;
    alc -= aligned - size;

    backtrace_free(state, (char*) vec->base + aligned, alc,
                   error_callback, data);
    vec->alc = 0;
    return 1;
}
