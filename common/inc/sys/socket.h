#ifndef _SYS_SOCKET_H_
#define _SYS_SOCKET_H_

#include <sys/_types.h>
#include <sys/sockaddr.h>

/* Type for length arguments in socket calls.  */
#ifndef __socklen_t_defined
typedef __socklen_t socklen_t;
# define __socklen_t_defined
#endif

/* Structure describing a generic socket address.  */
struct sockaddr
{
    __SOCKADDR_COMMON (sa_);	/* Common data: address family and length.  */
    char sa_data[14];		/* Address data.  */
};

#endif