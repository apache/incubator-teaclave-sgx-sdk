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
#include <sys/socket.h>
#include <errno.h>

int u_socket_ocall(int * error, int domain, int ty, int protocol)
{
    int ret = socket(domain, ty, protocol);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_socketpair_ocall(int * error, int domain, int ty, int protocol, int sv[2])
{
    int ret = socketpair(domain, ty, protocol, sv);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_bind_ocall(int * error, int sockfd, const struct sockaddr * addr, socklen_t addrlen)
{
    int ret = bind(sockfd, addr, addrlen);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_listen_ocall(int * error, int sockfd, int backlog)
{
    int ret = listen(sockfd, backlog);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_accept_ocall(int * error,
                   int sockfd,
                   struct sockaddr * addr,
                   socklen_t addrlen_in,
                   socklen_t * addrlen_out)
{
    *addrlen_out = addrlen_in;
    int ret = accept(sockfd, addr, addrlen_out);
     if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_accept4_ocall(int * error,
                    int sockfd,
                    struct sockaddr * addr,
                    socklen_t addrlen_in,
                    socklen_t * addrlen_out,
                    int flags)
{
    *addrlen_out = addrlen_in;
    int ret = accept4(sockfd, addr, addrlen_out, flags);
     if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_connect_ocall(int * error, int sockfd, const struct sockaddr * addr, socklen_t addrlen)
{
    int ret = connect(sockfd, addr, addrlen);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_recv_ocall(int * error, int sockfd, void * buf, size_t len, int flags)
{
    ssize_t ret = recv(sockfd, buf, len, flags);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_recvfrom_ocall(int * error,
                         int sockfd,
                         void * buf,
                         size_t len,
                         int flags,
                         struct sockaddr * src_addr,
                         socklen_t addrlen_in,
                         socklen_t * addrlen_out)
{
    *addrlen_out = addrlen_in;
    ssize_t ret = recvfrom(sockfd, buf, len, flags, src_addr, addrlen_out);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_recvmsg_ocall(int * error, int sockfd, struct msghdr * msg, int flags)
{
    ssize_t ret = recvmsg(sockfd, msg, flags);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_send_ocall(int * error, int sockfd, const void * buf, size_t len, int flags)
{
    ssize_t ret = send(sockfd, buf, len, flags);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_sendto_ocall(int * error,
                       int sockfd,
                       const void * buf,
                       size_t len,
                       int flags,
                       const struct sockaddr * dest_addr,
                       socklen_t addrlen)
{
    ssize_t ret = sendto(sockfd, buf, len, flags, dest_addr, addrlen);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

ssize_t u_sendmsg_ocall(int * error, int sockfd, const struct msghdr * msg, int flags)
{
    ssize_t ret = sendmsg(sockfd, msg, flags);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_getsockopt_ocall(int * error,
                       int sockfd,
                       int level,
                       int optname,
                       void * optval,
                       socklen_t optlen_in,
                       socklen_t * optlen_out)
{
    *optlen_out = optlen_in;
    int ret = getsockopt(sockfd, level, optname, optval, optlen_out);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_setsockopt_ocall(int * error, int sockfd, int level, int optname, const void * optval, socklen_t optlen)
{
    int ret = setsockopt(sockfd, level, optname, optval, optlen);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_getsockname_ocall(int * error, int sockfd, struct sockaddr * addr, socklen_t addrlen_in, socklen_t * addrlen_out)
{
    *addrlen_out = addrlen_in;
    int ret = getsockname(sockfd, addr, addrlen_out);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_getpeername_ocall(int * error, int sockfd, struct sockaddr * addr, socklen_t addrlen_in, socklen_t * addrlen_out)
{
    *addrlen_out = addrlen_in;
    int ret = getpeername(sockfd, addr, addrlen_out);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_shutdown_ocall(int * error, int sockfd, int how)
{
    int ret = shutdown(sockfd, how);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}