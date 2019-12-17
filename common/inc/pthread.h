#ifndef _SYS_THREAD_H_
#define _SYS_THREAD_H_

/* Thread identifiers.  The structure of the attribute type is not
   exposed on purpose.  */
typedef unsigned long int pthread_t;

#if defined __x86_64__ && !defined __ILP32__
# define __WORDSIZE 64
#else
# define __WORDSIZE 32
#define __WORDSIZE32_SIZE_ULONG     0
#define __WORDSIZE32_PTRDIFF_LONG   0
#endif

#ifdef __x86_64__
# if __WORDSIZE == 64
#  define __SIZEOF_PTHREAD_ATTR_T   56
# else
#  define __SIZEOF_PTHREAD_ATTR_T   32
#endif

union pthread_attr_t
{
    char __size[__SIZEOF_PTHREAD_ATTR_T];
    long int __align;
};
#ifndef __have_pthread_attr_t
typedef union pthread_attr_t pthread_attr_t;
# define __have_pthread_attr_t 1
#endif

#endif
#endif
