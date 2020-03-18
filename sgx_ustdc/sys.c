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

#define _GNU_SOURCE
#include <unistd.h>
#include <errno.h>
#include <sys/prctl.h>
#include <sched.h>

long u_sysconf_ocall(int *error, int name)
{
    long ret = sysconf(name);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_prctl_ocall(int *error,
                  int option,
                  unsigned long arg2,
                  unsigned long arg3,
                  unsigned long arg4,
                  unsigned long arg5)
{
    int ret = prctl(option, arg2, arg3, arg4, arg5);
    if (error) {
        *error = ret < 0  ? errno : 0;
    }
    return ret;
}

int u_sched_setaffinity_ocall(int *error, pid_t pid, size_t cpusetsize, cpu_set_t *mask)
{
    int ret = sched_setaffinity(pid, cpusetsize, mask);
    if (error) {
        *error = ret < 0  ? errno : 0;
    }
    return ret;
}

int u_sched_getaffinity_ocall(int *error, pid_t pid, size_t cpusetsize, cpu_set_t *mask)
{
    int ret = sched_getaffinity(pid, cpusetsize, mask);
    if (error) {
        *error = ret < 0  ? errno : 0;
    }
    return ret;
}