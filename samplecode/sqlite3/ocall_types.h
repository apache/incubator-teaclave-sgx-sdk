#ifndef _OCALL_TYPES_H_
#define _OCALL_TYPES_H_

// Divide system definitions into trusted and untrusted part for ocalls type declarations
#ifdef SGX_UNTRUSTED
// For untrusted part take standard library headers
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>

#else
// For trusted part copy required standard library declarations from stdlib headers

// For ocall_interface.c do not redefine these types, otherwise define
#ifndef DO_NOT_REDEFINE_FOR_OCALL

typedef unsigned long int __dev_t;
typedef unsigned int __uid_t;
typedef unsigned int __gid_t;
typedef unsigned long int __ino_t;
typedef unsigned long int __ino64_t;
typedef unsigned int __mode_t;
typedef unsigned int mode_t;
typedef unsigned long int __nlink_t;
typedef long int __off_t;
typedef long int __off64_t;
typedef int __pid_t;
typedef long int __clock_t;
typedef unsigned long int __rlim_t;
typedef unsigned long int __rlim64_t;
typedef unsigned int __id_t;
typedef long int __time_t;
typedef unsigned int __useconds_t;
typedef long int __suseconds_t;
typedef long int __blksize_t;
typedef long int __blkcnt_t;
typedef long int __blkcnt64_t;
typedef __off_t off_t;
typedef long int __syscall_slong_t;

struct stat
{
    __dev_t st_dev;
    __ino_t st_ino;
    __nlink_t st_nlink;
    __mode_t st_mode;
    __uid_t st_uid;
    __gid_t st_gid;
    int __pad0;
    __dev_t st_rdev;
    __off_t st_size;
    __blksize_t st_blksize;
    __blkcnt_t st_blocks;
    __time_t st_atim;
    __time_t st_mtim;
    __time_t st_ctim;
    __syscall_slong_t __glibc_reserved[3];
};

#endif // DO_NOT_REDEFINE_FOR_OCALL_INTERFACE

#endif // SGX_UNTRUSTED

#endif // _OCALL_TYPES_H_
