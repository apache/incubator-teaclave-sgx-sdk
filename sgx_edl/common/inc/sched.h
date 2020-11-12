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

#ifndef _SCHED_H
#define _SCHED_H
#include <sys/_types.h>

typedef struct {
  unsigned long __bits[128/sizeof(long)];
} cpu_set_t;

#define __CPU_op_S(i, size, set, op) ( (i)/8U >= (size) ? 0 : \
    (((unsigned long *)(set))[(i)/8/sizeof(long)] op (1UL<<((i)%(8*sizeof(long))))) )

#define CPU_SET_S(i, size, set) __CPU_op_S(i, size, set, |=)
#define CPU_CLR_S(i, size, set) __CPU_op_S(i, size, set, &=~)
#define CPU_ISSET_S(i, size, set) __CPU_op_S(i, size, set, &)

#define __CPU_op_func_S(func, op) \
static __inline void __CPU_##func##_S(size_t __size, cpu_set_t *__dest, \
    const cpu_set_t *__src1, const cpu_set_t *__src2) \
{ \
    size_t __i; \
    for (__i=0; __i<__size/sizeof(long); __i++) \
        ((unsigned long *)__dest)[__i] = ((unsigned long *)__src1)[__i] \
            op ((unsigned long *)__src2)[__i] ; \
}

__CPU_op_func_S(AND, &)
__CPU_op_func_S(OR, |)
__CPU_op_func_S(XOR, ^)

#define CPU_AND_S(a,b,c,d) __CPU_AND_S(a,b,c,d)
#define CPU_OR_S(a,b,c,d) __CPU_OR_S(a,b,c,d)
#define CPU_XOR_S(a,b,c,d) __CPU_XOR_S(a,b,c,d)

typedef __pid_t   pid_t;

#endif
