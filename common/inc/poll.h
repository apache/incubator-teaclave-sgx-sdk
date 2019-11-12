#ifndef _POLL_H_
#define _POLL_H_

/* Type used for the number of file descriptors.  */
typedef unsigned long int nfds_t;

/* Data structure describing a polling request.  */
struct pollfd
{
    int fd;             /* File descriptor to poll.  */
    short int events;   /* Types of events poller cares about.  */
    short int revents;  /* Types of events that actually occurred.  */
};

#endif
