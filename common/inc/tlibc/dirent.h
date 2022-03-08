//
// Copyright © 2005-2020 Rich Felker, et al.
// Licensed under the MIT license.s
//

/* Copyright © 2005-2020 Rich Felker, et al.

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE. */

#ifndef _DIRENT_H_
#define _DIRENT_H_

#include <sys/cdefs.h>
#include <sys/types.h>

struct __dirstream
{
    off_t tell;
    int fd;
    int buf_pos;
    int buf_end;
    volatile int lock[1];
    /* Any changes to this struct must preserve the property:
     * offsetof(struct __dirent, buf) % sizeof(off_t) == 0 */
    char buf[2048];
};

typedef struct __dirstream DIR;

struct dirent {
    __ino_t d_ino;
    __off_t d_off;
    unsigned short d_reclen;
    unsigned char d_type;
    char d_name[256];
};

struct dirent64 {
    __ino64_t d_ino;
    __off64_t d_off;
    unsigned short d_reclen;
    unsigned char d_type;
    char d_name[256];
};

#define d_fileno	d_ino

__BEGIN_DECLS

DIR *fdopendir(int);
DIR *opendir(const char *);
int readdir64_r(DIR *__restrict, struct dirent64 *__restrict, struct dirent64 **__restrict);
int closedir(DIR *);
int dirfd(DIR *);

__END_DECLS

#endif /* _DIRENT_H_ */
