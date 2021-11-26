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

#define _LARGEFILE64_SOURCE

#include <sys/types.h>
#include <sys/ioctl.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <stdio.h>
#include <errno.h>
#include <fcntl.h>
#include <unistd.h>
#include <limits.h>
#include <stdlib.h>
#include <dirent.h>

int u_open_ocall(int *error, const char *pathname, int flags)
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

int u_fstat_ocall(int *error, int fd, struct stat *buf)
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

int u_lstat_ocall(int *error, const char *path, struct stat *buf)
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

off_t u_lseek_ocall(int *error, int fd, off_t offset, int whence)
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

int u_linkat_ocall(int *error, int olddirfd, const char *oldpath, int newdirfd, const char *newpath, int flags)
{
    int ret = linkat(olddirfd, oldpath, newdirfd, newpath, flags);
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

int u_mkdir_ocall(int *error, const char *pathname, mode_t mode)
{
    int ret = mkdir(pathname, mode);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_rmdir_ocall(int *error, const char *pathname)
{
     int ret = rmdir(pathname);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

void *u_opendir_ocall(int *error, const char *pathname)
{
    DIR *ret = opendir(pathname);
    if (error) {
        *error = ret == NULL ? errno : 0;
    }
    return ret;
}

int u_readdir64_r_ocall(DIR *dirp, struct dirent64 *entry, struct dirent64 **result)
{
    return readdir64_r(dirp, entry, result);
}

int u_closedir_ocall(int *error, DIR *dirp)
{
    int ret = closedir(dirp);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_dirfd_ocall(int *error, DIR *dirp) 
{
    int ret = dirfd(dirp);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_fstatat64_ocall(int *error,
                      int dirfd,
                      const char *pathname,
                      struct stat64 *buf,
                      int flags)
{
    int ret = fstatat64(dirfd, pathname, buf, flags);
     if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}