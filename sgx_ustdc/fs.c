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

int u_fs_open64_ocall(int *error, const char *path, int oflag, int mode)
{
    int ret = open64(path, oflag, mode);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_fs_read_ocall(int * error, int fd, void * buf, size_t count)
{
    ssize_t ret = read(fd, buf, count);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_fs_pread64_ocall(int * error, int fd, void * buf, size_t count, off64_t offset)
{
    ssize_t ret = pread64(fd, buf, count, offset);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_fs_write_ocall(int * error, int fd, const void * buf, size_t count)
{
    ssize_t ret = write(fd, buf, count);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_fs_pwrite64_ocall(int * error, int fd, const void * buf, size_t count, off64_t offset)
{
    ssize_t ret = pwrite64(fd, buf, count, offset);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_close_ocall(int * error, int fd)
{
    int ret = close(fd);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_fcntl_arg0_ocall(int * error, int fd, int cmd)
{
    int ret = fcntl(fd, cmd);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_fcntl_arg1_ocall(int * error, int fd, int cmd, int arg)
{
    int ret = fcntl(fd, cmd, arg);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_ioctl_arg0_ocall(int * error, int fd, int request)
{
    int ret = ioctl(fd, request);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_ioctl_arg1_ocall(int * error, int fd, int request, int * arg)
{
    int ret = ioctl(fd, request, arg);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_fstat64_ocall(int *error, int fd, struct stat64 *buf)
{
    int ret = fstat64(fd, buf);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_fsync_ocall(int *error, int fd)
{
    int ret = fsync(fd);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_fdatasync_ocall(int *error, int fd)
{
    int ret = fdatasync(fd);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_ftruncate64_ocall(int *error, int fd, off64_t length)
{
    int ret = ftruncate64(fd, length);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

off64_t u_fs_lseek64_ocall(int *error, int fd, off64_t offset, int whence)
{
    off64_t ret = lseek64(fd, offset, whence);
    if (error) {
        *error = ret == (off64_t)-1 ? errno : 0;
    }
    return ret;
}

int u_fs_fchmod_ocall(int *error, int fd, mode_t mode)
{
    int ret = fchmod(fd, mode);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_unlink_ocall(int *error, const char *pathname)
{
    int ret = unlink(pathname);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_link_ocall(int *error, const char *oldpath, const char *newpath)
{
    int ret = link(oldpath, newpath);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_rename_ocall(int *error, const char *oldpath, const char *newpath)
{
    int ret = rename(oldpath, newpath);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_chmod_ocall(int *error, const char *path, mode_t mode)
{
    int ret = chmod(path, mode);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_fs_readlink_ocall(int *error, const char *path, char *buf, size_t bufsz)
{
    ssize_t ret = readlink(path, buf, bufsz);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_symlink_ocall(int *error, const char *path1, const char *path2)
{
    int ret = symlink(path1, path2);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_stat64_ocall(int *error, const char *path, struct stat64 *buf)
{
    int ret = stat64(path, buf);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fs_lstat64_ocall(int *error, const char *path, struct stat64 *buf)
{
    int ret = lstat64(path, buf);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

char * u_fs_realpath_ocall(int *error, const char *pathname)
{
    char *ret = realpath(pathname, NULL);
    if (error) {
        *error = ret == NULL ? errno : 0;
    }
    return ret;
}

void u_fs_free_ocall(void *p)
{
    free(p);
}
