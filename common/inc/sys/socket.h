/* Copyright (C) 1996-2018 Free Software Foundation, Inc.
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

#ifndef _SYS_SOCKET_H_
#define _SYS_SOCKET_H_

#include <sys/_types.h>
#include <sys/sockaddr.h>
#include <sys/uio.h>

/* Type for length arguments in socket calls.  */
#ifndef __socklen_t_defined
typedef __socklen_t socklen_t;
# define __socklen_t_defined
#endif

/* Structure describing a generic socket address.  */
struct sockaddr
{
    __SOCKADDR_COMMON (sa_);    /* Common data: address family and length.  */
    char sa_data[14];           /* Address data.  */
};

struct msghdr
{
    void *msg_name;         /* Address to send to/receive from.  */
    socklen_t msg_namelen;  /* Length of address data.  */

    struct iovec *msg_iov;  /* Vector of data to send/receive into.  */
    size_t msg_iovlen;      /* Number of elements in the vector.  */

    void *msg_control;      /* Ancillary data (eg BSD filedesc passing). */
    size_t msg_controllen;  /* Ancillary data buffer length.
                            !! The type should be socklen_t but the
                            definition of the kernel is incompatible
                            with this.  */
    int msg_flags;          /* Flags on received message.  */
};

#endif
