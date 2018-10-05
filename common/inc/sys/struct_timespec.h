#ifndef _SYS_TIMESPEC_H_
#define _SYS_TIMESPEC_H_

#include <sys/_types.h>

/* POSIX.1b structure for a time value.  This is like a `struct timeval' but
   has nanoseconds instead of microseconds.  */
struct timespec
{
	__time_t 	tv_sec;		/* Seconds.  */
	long 	   	tv_nsec;	/* Nanoseconds.  */
};

#endif
