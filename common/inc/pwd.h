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

/*
 *	POSIX Standard: 9.2.2 User Database Access	<pwd.h>
 */

#ifndef _PWD_H
#define _PWD_H

struct passwd
{
    char *pw_name;      /* Username.  */
    char *pw_passwd;    /* Password.  */
    __uid_t pw_uid;     /* User ID.  */
    __gid_t pw_gid;     /* Group ID.  */
    char *pw_gecos;     /* Real name.  */
    char *pw_dir;       /* Home directory.  */
    char *pw_shell;     /* Shell program.  */
};

#endif
