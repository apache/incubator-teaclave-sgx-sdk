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

#include <sys/types.h>
#include <sys/socket.h>
#include <netdb.h>
#include <errno.h>

int u_getaddrinfo_ocall(int *error,
                        const char *node,
                        const char *service,
                        const struct addrinfo *hints,
                        struct addrinfo **res)
{
    int ret = getaddrinfo(node, service, hints, res);
    if (error) {
        *error = ret == EAI_SYSTEM ? errno : 0;
    }
    return ret;
}

void u_freeaddrinfo_ocall(struct addrinfo *res)
{
    return freeaddrinfo(res);
}

const char *u_gai_strerror_ocall(int errcode)
{
    return gai_strerror(errcode);
}