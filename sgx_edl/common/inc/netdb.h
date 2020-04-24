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

/* All data returned by the network data base library are supplied in
   host order and returned in network order (suitable for use in
   system calls).  */

#ifndef _NETDB_H
#define _NETDB_H

struct addrinfo
{
    int ai_flags;               /* Input flags.  */
    int ai_family;              /* Protocol family for socket.  */
    int ai_socktype;            /* Socket type.  */
    int ai_protocol;            /* Protocol for socket.  */
    socklen_t ai_addrlen;       /* Length of socket address.  */
    struct sockaddr *ai_addr;   /* Socket address for socket.  */
    char *ai_canonname;         /* Canonical name for service location.  */
    struct addrinfo *ai_next;   /* Pointer to next in list.  */
};

#endif
