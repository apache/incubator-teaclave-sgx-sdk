// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <sys/types.h>
#include <sys/epoll.h>
#include <poll.h>
#include <errno.h>

int u_poll_ocall(int * error, struct pollfd * fds, nfds_t nfds, int timeout)
{
    int ret = poll(fds, nfds, timeout);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_epoll_create1_ocall(int * error, int flags)
{
    int ret = epoll_create1(flags);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_epoll_ctl_ocall(int * error, int epfd, int op, int fd, struct epoll_event * event)
{
    int ret = epoll_ctl(epfd, op, fd, event);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_epoll_wait_ocall(int * error, int epfd, struct epoll_event * events, int maxevents, int timeout)
{
    int ret = epoll_wait(epfd, events, maxevents, timeout);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}