/* Copyright (C) 1991-2018 Free Software Foundation, Inc.
   This file is part of the GNU C Library.

   The GNU C Library is free software; you can redistribute it and/or
   modify it under the terms of the GNU Lesser General Public
   License as published by the Free Software Foundation; either
   version 2.1 of the License, or (at your option) any later version.

   The GNU C Library is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Lesser General Public License for more details.

   You should have received a copy of the GNU Lesser General Public
   License along with the GNU C Library; if not, see
   <http://www.gnu.org/licenses/>.  */

#ifndef _SIGNAL_H
#define _SIGNAL_H

#define _SIGSET_NWORDS (1024 / (8 * sizeof (unsigned long int)))
typedef struct
{
    unsigned long int __val[_SIGSET_NWORDS];
} __sigset_t;

typedef __sigset_t sigset_t;

struct sigaction
{
    /* Signal handler.  */
#if defined __USE_POSIX199309 || defined __USE_XOPEN_EXTENDED
    union
    {
        /* Used if SA_SIGINFO is not set.  */
        void (*sa_handler) (int);
        /* Used if SA_SIGINFO is set.  */
        void (*sa_sigaction) (int, siginfo_t *, void *);
    }
    __sigaction_handler;
#define sa_handler	__sigaction_handler.sa_handler
#define sa_sigaction __sigaction_handler.sa_sigaction
#else
    void (*sa_handler) (int);
#endif

    /* Additional set of signals to be blocked.  */
    __sigset_t sa_mask;

    /* Special flags.  */
    int sa_flags;

    /* Restore handler.  */
    void (*sa_restorer) (void);
};

#define __SI_MAX_SIZE	128
#define __SI_PAD_SIZE	((__SI_MAX_SIZE / sizeof (int)) - 4)

typedef struct
{
    int si_signo;   /* Signal number.  */

    int si_errno;   /* If non-zero, an errno value associated with
                    this signal, as defined in <errno.h>.  */
    int si_code;    /* Signal code.  */

    int __pad0;
    int _pad[__SI_PAD_SIZE];
} siginfo_t;

#endif