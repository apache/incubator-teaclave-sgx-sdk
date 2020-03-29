// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <sys/types.h>
#include <sys/epoll.h>
#include <poll.h>
#include <errno.h>

int u_poll_ocall(int *error, struct pollfd *fds, nfds_t nfds, int timeout)
{
    int ret = poll(fds, nfds, timeout);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_epoll_create1_ocall(int *error, int flags)
{
    int ret = epoll_create1(flags);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_epoll_ctl_ocall(int *error, int epfd, int op, int fd, struct epoll_event *event)
{
    int ret = epoll_ctl(epfd, op, fd, event);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_epoll_wait_ocall(int *error, int epfd, struct epoll_event *events, int maxevents, int timeout)
{
    int ret = epoll_wait(epfd, events, maxevents, timeout);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}