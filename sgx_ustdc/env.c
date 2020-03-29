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

#include <unistd.h>
#include <sys/types.h>
#include <stdlib.h>
#include <errno.h>
#include <pwd.h>
extern char **environ;

uid_t u_getuid_ocall()
{
    return getuid();
}

char **u_environ_ocall()
{
    return environ;
}

char *u_getenv_ocall(const char *name)
{
    return getenv(name);
}

int u_setenv_ocall(int *error, const char *name, const char *value, int overwrite)
{
    int ret = setenv(name, value, overwrite);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_unsetenv_ocall(int *error, const char *name)
{
    int ret = unsetenv(name);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

char *u_getcwd_ocall(int *error, char *buf, size_t size)
{
    char *ret = getcwd(buf, size);
    if (error) {
        *error = ret == NULL ? errno : 0;
    }
    return ret;
}

int u_chdir_ocall(int *error, const char *dir)
{
    int ret = chdir(dir);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_getpwuid_r_ocall(uid_t uid,
                       struct passwd *pwd,
                       char *buf,
                       size_t buflen,
                       struct passwd **passwd_result)
{
    int ret = getpwuid_r(uid, pwd, buf, buflen, passwd_result);
    if (ret == 0 && *passwd_result != NULL) {
        pwd->pw_name = pwd->pw_name ? (char *)(pwd->pw_name - buf) : (char *)-1;
        pwd->pw_passwd = pwd->pw_passwd ? (char *)(pwd->pw_passwd - buf) : (char *)-1;
        pwd->pw_gecos = pwd->pw_gecos ? (char *)(pwd->pw_gecos - buf) : (char *)-1;
        pwd->pw_dir = pwd->pw_dir ? (char *)(pwd->pw_dir - buf) : (char *)-1;
        pwd->pw_shell = pwd->pw_shell ? (char *)(pwd->pw_shell - buf) : (char *)-1;
    }
    return ret;
}