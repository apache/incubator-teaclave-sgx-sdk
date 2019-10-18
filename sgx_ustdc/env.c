// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

#include <sys/types.h>
#include <stdlib.h>
#include <errno.h>
#include <pwd.h>
extern char **environ;

uid_t u_getuid_ocall()
{
    return getuid();
}

char ** u_environ_ocall()
{
    return environ;
}

char * u_getenv_ocall(const char * name)
{
    return getenv(name);
}

int u_setenv_ocall(int * error, const char * name, const char * value, int overwrite)
{
    int ret = setenv(name, value, overwrite);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_unsetenv_ocall(int * error, const char * name)
{
    int ret = unsetenv(name);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

char* u_getcwd_ocall(int *error, char *buf, size_t size)
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
        pwd->pw_name = pwd->pw_name ? pwd->pw_name - buf : -1;
        pwd->pw_passwd = pwd->pw_passwd ? pwd->pw_passwd - buf : -1;
        pwd->pw_gecos = pwd->pw_gecos ? pwd->pw_gecos - buf : -1;
        pwd->pw_dir = pwd->pw_dir ? pwd->pw_dir - buf : -1;
        pwd->pw_shell = pwd->pw_shell ? pwd->pw_shell - buf : -1;
    }
    return ret;
}