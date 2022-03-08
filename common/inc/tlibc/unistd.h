/*  $OpenBSD: unistd.h,v 1.62 2008/06/25 14:58:54 millert Exp $ */
/*  $NetBSD: unistd.h,v 1.26.4.1 1996/05/28 02:31:51 mrg Exp $  */

/*-
 * Copyright (c) 1991 The Regents of the University of California.
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. Neither the name of the University nor the names of its contributors
 *    may be used to endorse or promote products derived from this software
 *    without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE REGENTS OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 *
 *  @(#)unistd.h    5.13 (Berkeley) 6/17/91
 */

#ifndef _UNISTD_H_
#define	_UNISTD_H_

#include <sys/cdefs.h>
#include <sys/types.h>

__BEGIN_DECLS

int getpagesize(void);

void * _TLIBC_CDECL_ sbrk(intptr_t);

/*
 * Deprecated Non-C99.
 */
_TLIBC_DEPRECATED_FUNCTION_(int _TLIBC_CDECL_, execl, const char *, const char *, ...);
_TLIBC_DEPRECATED_FUNCTION_(int _TLIBC_CDECL_, execlp, const char *, const char *, ...);
_TLIBC_DEPRECATED_FUNCTION_(int _TLIBC_CDECL_, execle, const char *, const char *, ...);
_TLIBC_DEPRECATED_FUNCTION_(int _TLIBC_CDECL_, execv, const char *, char * const *);
_TLIBC_DEPRECATED_FUNCTION_(int _TLIBC_CDECL_, execve, const char *, char * const *, char * const *);
_TLIBC_DEPRECATED_FUNCTION_(int _TLIBC_CDECL_, execvp, const char *, char * const *);

//_TLIBC_DEPRECATED_FUNCTION_(pid_t _TLIBC_CDECL_, fork, void); /* no pid_t */

// ocall
uid_t getuid(void);
gid_t getgid(void);
pid_t getpid(void);
int isatty(int);
int  chdir(const char *);
char * getcwd(char *, size_t);

int pipe2(int [2], int);
int close(int);
int dup(int);
int fsync(int);
int fdatasync(int);
off_t lseek(int, off_t, int);
off64_t lseek(int, off64_t, int);
int ftruncate(int, off_t);
int ftruncate64(int, off64_t);
int truncate(const char *, off_t);
int truncate64(const char *, off64_t);

int link(const char *, const char *);
int linkat(int, const char *, int, const char *, int);
int unlink(const char *);
int unlinkat(int, const char *, int);
int symlink(const char *, const char *);
ssize_t readlink(const char *__restrict, char *__restrict, size_t);
int rmdir(const char *);

ssize_t read(int, void *, size_t);
ssize_t write(int, const void *, size_t);
ssize_t pread64(int, void *, size_t, off64_t);
ssize_t pwrite64(int, const void *, size_t, off64_t);

ssize_t copy_file_range(int, off64_t *, int, off64_t *, size_t, unsigned);

long sysconf(int);

__END_DECLS

#endif /* _UNISTD_H_ */
