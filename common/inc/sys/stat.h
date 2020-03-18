/* Copyright (C) 1999-2017 Free Software Foundation, Inc.
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


#ifndef _SYS_STAT_H
#define _SYS_STAT_H

#include <sys/_types.h>
#include <sys/types.h>
#include <time.h>

/* Versions of the `struct stat' data structure.  */
#ifndef __x86_64__
# define _STAT_VER_LINUX_OLD    1
# define _STAT_VER_KERNEL       1
# define _STAT_VER_SVR4         2
# define _STAT_VER_LINUX        3

/* i386 versions of the `xmknod' interface.  */
# define _MKNOD_VER_LINUX       1
# define _MKNOD_VER_SVR4        2
# define _MKNOD_VER     _MKNOD_VER_LINUX /* The bits defined below.  */
#else
# define _STAT_VER_KERNEL       0
# define _STAT_VER_LINUX        1

/* x86-64 versions of the `xmknod' interface.  */
# define _MKNOD_VER_LINUX   0
#endif

#define _STAT_VER       _STAT_VER_LINUX

#ifndef _DEV_T_DEFINED_
#define _DEV_T_DEFINED_
typedef __dev_t     dev_t;
#endif

#ifndef _INO_T_DEFINED_
#define _INO_T_DEFINED_
typedef __ino_t     ino_t;
typedef __ino64_t   ino64_t;
#endif

#ifndef _MODE_T_DEFINED_
#define _MODE_T_DEFINED_
typedef __mode_t    mode_t;
#endif

#ifndef _NLINK_T_DEFINED_
#define _NLINK_T_DEFINED_
typedef __nlink_t   nlink_t;
#endif

#ifndef _UID_T_DEFINED_
#define _UID_T_DEFINED_
typedef __uid_t     uid_t;
#endif

#ifndef _GID_T_DEFINED_
#define _GID_T_DEFINED_
typedef __gid_t     gid_t;
#endif

#ifndef _BLKSIZE_T_DEFINED_
#define _BLKSIZE_T_DEFINED_
typedef __blksize_t blksize_t;
#endif

#ifndef _BLKCNT_T_DEFINED_
#define _BLKCNT_T_DEFINED_
typedef __blkcnt_t      blkcnt_t;
typedef __blkcnt64_t    blkcnt64_t;
#endif


struct stat
{
    dev_t st_dev;       /* Device.  */
#ifndef __x86_64__
    unsigned short int __pad1;
#endif
#if defined __x86_64__ || !defined __USE_FILE_OFFSET64
    ino_t st_ino;           /* File serial number.	*/
#else
    ino_t __st_ino;         /* 32bit file serial number.	*/
#endif
#ifndef __x86_64__
    mode_t st_mode;         /* File mode.  */
    nlink_t st_nlink;       /* Link count.  */
#else
    nlink_t st_nlink;       /* Link count.  */
    mode_t st_mode;         /* File mode.  */
#endif
    uid_t st_uid;           /* User ID of the file's owner.	*/
    gid_t st_gid;           /* Group ID of the file's group.*/
#ifdef __x86_64__
    int __pad0;
#endif
    dev_t st_rdev;          /* Device number, if device.  */
#ifndef __x86_64__
    unsigned short int __pad2;
#endif
#if defined __x86_64__ || !defined __USE_FILE_OFFSET64
    off_t st_size;          /* Size of file, in bytes.  */
#else
    off64_t st_size;        /* Size of file, in bytes.  */
#endif
    blksize_t st_blksize;   /* Optimal block size for I/O.  */
#if defined __x86_64__  || !defined __USE_FILE_OFFSET64
    blkcnt_t st_blocks;     /* Number 512-byte blocks allocated. */
#else
    blkcnt64_t st_blocks;   /* Number 512-byte blocks allocated. */
#endif

    time_t st_atime;            /* Time of last access.  */
    unsigned long int  st_atimensec;    /* Nscecs of last access.  */
    time_t st_mtime;			/* Time of last modification.  */
    unsigned long int  st_mtimensec;    /* Nsecs of last modification.  */
    time_t st_ctime;            /* Time of last status change.  */
    unsigned long int  st_ctimensec;    /* Nsecs of last status change.  */
#ifdef __x86_64__
    long int  __glibc_reserved[3];
#else
# ifndef __USE_FILE_OFFSET64
    unsigned long int __glibc_reserved4;
    unsigned long int __glibc_reserved5;
# else
    ino64_t st_ino;         /* File serial number.	*/
# endif
#endif
};

struct stat64
{
    dev_t st_dev;           /* Device.  */
# ifdef __x86_64__
    ino64_t st_ino;         /* File serial number.  */
    nlink_t st_nlink;       /* Link count.  */
    mode_t st_mode;         /* File mode.  */
# else
    unsigned int __pad1;
    ino_t __st_ino;         /* 32bit file serial number.	*/
    mode_t st_mode;         /* File mode.  */
    nlink_t st_nlink;       /* Link count.  */
# endif
    uid_t st_uid;           /* User ID of the file's owner.	*/
    gid_t st_gid;           /* Group ID of the file's group.*/
# ifdef __x86_64__
    int __pad0;
    dev_t st_rdev;          /* Device number, if device.  */
    off_t st_size;          /* Size of file, in bytes.  */
# else
    dev_t st_rdev;          /* Device number, if device.  */
    unsigned int __pad2;
    off64_t st_size;        /* Size of file, in bytes.  */
# endif
    blksize_t st_blksize;   /* Optimal block size for I/O.  */
    blkcnt64_t st_blocks;   /* Nr. 512-byte blocks allocated.  */
# ifdef __USE_XOPEN2K8
    /* Nanosecond resolution timestamps are stored in a format
       equivalent to 'struct timespec'.  This is the type used
       whenever possible but the Unix namespace rules do not allow the
       identifier 'timespec' to appear in the <sys/stat.h> header.
       Therefore we have to handle the use of this header in strictly
       standard-compliant sources special.  */
    struct timespec st_atim;        /* Time of last access.  */
    struct timespec st_mtim;        /* Time of last modification.  */
    struct timespec st_ctim;        /* Time of last status change.  */
# else
    time_t st_atime;            /* Time of last access.  */
    unsigned long int st_atimensec; /* Nscecs of last access.  */
    time_t st_mtime;            /* Time of last modification.  */
    unsigned long int st_mtimensec; /* Nsecs of last modification.  */
    time_t st_ctime;            /* Time of last status change.  */
    unsigned long int st_ctimensec;	/* Nsecs of last status change.  */
# endif
# ifdef __x86_64__
    long int __glibc_reserved[3];
# else
    ino64_t st_ino;             /* File serial number.		*/
# endif
 };

/* Tell code we have these members.  */
#define _STATBUF_ST_BLKSIZE
#define _STATBUF_ST_RDEV
/* Nanosecond resolution time values are supported.  */
#define _STATBUF_ST_NSEC

/* Encoding of the file mode.  */

#define S_IFMT  0170000	/* These bits determine file type.  */

/* File types.  */
#define S_IFDIR 0040000 /* Directory.  */
#define S_IFCHR 0020000 /* Character device.  */
#define S_IFBLK 0060000 /* Block device.  */
#define S_IFREG 0100000 /* Regular file.  */
#define S_IFIFO 0010000 /* FIFO.  */
#define S_IFLNK 0120000 /* Symbolic link.  */
#define S_IFSOCK    0140000 /* Socket.  */

/* POSIX.1b objects.  Note that these macros always evaluate to zero.  But
   they do it by enforcing the correct use of the macros.  */
#define S_TYPEISMQ(buf)     ((buf)->st_mode - (buf)->st_mode)
#define S_TYPEISSEM(buf)    ((buf)->st_mode - (buf)->st_mode)
#define S_TYPEISSHM(buf)    ((buf)->st_mode - (buf)->st_mode)

/* Protection bits.  */

#define S_ISUID 04000   /* Set user ID on execution.  */
#define S_ISGID 02000   /* Set group ID on execution.  */
#define S_ISVTX 01000   /* Save swapped text after use (sticky).  */
#define S_IREAD 0400    /* Read by owner.  */
#define S_IWRITE    0200    /* Write by owner.  */
#define S_IEXEC 0100    /* Execute by owner.  */

#ifdef __USE_ATFILE
# define UTIME_NOW  ((1l << 30) - 1l)
# define UTIME_OMIT ((1l << 30) - 2l)
#endif

#define S_ISTYPE(mode, mask)    (((mode) & __S_IFMT) == (mask))

#define S_ISDIR(mode)   S_ISTYPE((mode), S_IFDIR)
#define S_ISCHR(mode)   S_ISTYPE((mode), S_IFCHR)
#define S_ISBLK(mode)   S_ISTYPE((mode), S_IFBLK)
#define S_ISREG(mode)   S_ISTYPE((mode), S_IFREG)
#define S_ISFIFO(mode)  S_ISTYPE((mode), S_IFIFO)
#define S_ISLNK(mode)   S_ISTYPE((mode), S_IFLNK)

#endif  /* bits/stat.h */
