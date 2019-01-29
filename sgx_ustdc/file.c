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
#include <sys/stat.h>
#include <stdio.h>
#include <errno.h>
#include <fcntl.h>
#include <unistd.h>
#include <limits.h>
#include <stdlib.h>

int u_open_ocall(int * error, const char * pathname, int flags)
{
    int ret = open(pathname, flags);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_open64_ocall(int *error, const char *path, int oflag, int mode)
{
    int ret = open64(path, oflag, mode);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fstat_ocall(int * error, int fd, struct stat * buf)
{
    int ret = fstat(fd, buf);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fstat64_ocall(int *error, int fd, struct stat64 *buf)
{
    int ret = fstat64(fd, buf);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_stat_ocall(int *error, const char *path, struct stat *buf)
{
    int ret = stat(path, buf);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_stat64_ocall(int *error, const char *path, struct stat64 *buf)
{
    int ret = stat64(path, buf);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_lstat_ocall(int * error, const char *path, struct stat * buf)
{
    int ret = lstat(path, buf);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_lstat64_ocall(int *error, const char *path, struct stat64 *buf)
{
    int ret = lstat64(path, buf);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

off_t u_lseek_ocall(int * error, int fd, off_t offset, int whence)
{
    off_t ret = lseek(fd, offset, whence);
    if (error) {
        *error = ret == (off_t)-1 ? errno : 0;
    }
    return ret;
}

off64_t u_lseek64_ocall(int *error, int fd, off64_t offset, int whence)
{
    off64_t ret = lseek64(fd, offset, whence);
    if (error) {
        *error = ret == (off64_t)-1 ? errno : 0;
    }
    return ret;
}

int u_ftruncate_ocall(int *error, int fd, off_t length)
{
    int ret = ftruncate(fd, length);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_ftruncate64_ocall(int *error, int fd, off64_t length)
{
    int ret = ftruncate64(fd, length);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_truncate_ocall(int *error, const char *path, off_t length)
{
    int ret = truncate(path, length);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_truncate64_ocall(int *error, const char *path, off64_t length)
{
    int ret = truncate64(path, length);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fsync_ocall(int *error, int fd)
{
    int ret = fsync(fd);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fdatasync_ocall(int *error, int fd)
{
    int ret = fdatasync(fd);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fchmod_ocall(int *error, int fd, mode_t mode)
{
    int ret = fchmod(fd, mode);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_unlink_ocall(int *error, const char *pathname)
{
    int ret = unlink(pathname);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_link_ocall(int *error, const char *oldpath, const char *newpath)
{
    int ret = link(oldpath, newpath);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_rename_ocall(int *error, const char *oldpath, const char *newpath)
{
    int ret = rename(oldpath, newpath);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_chmod_ocall(int *error, const char *path, mode_t mode)
{
    int ret = chmod(path, mode);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_readlink_ocall(int *error, const char *path, char *buf, size_t bufsz)
{
    ssize_t ret = readlink(path, buf, bufsz);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_symlink_ocall(int *error, const char *path1, const char *path2)
{
    int ret = symlink(path1, path2);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

char *u_realpath_ocall(int *error, const char *pathname)
{
    char *ret = realpath(pathname, NULL);
    if (error) {
        *error = ret == NULL ? errno : 0;
    }
    return ret;
}
