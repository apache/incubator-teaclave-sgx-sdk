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
#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdlib.h>
#include <string.h>
#include "inline-hashtab.h"
#include "spinlock.h"

typedef struct _sgx_signal_dispatcher_t {
    pthread_mutex_t lock;
    struct hashtab *signal_to_eid_set;
} sgx_signal_dispatcher_t;

typedef struct _key_value_t {
    int signum;
    unsigned long long enclave_id;
} key_value_t;

unsigned int t_signal_handler_ecall(unsigned long long eid, int* retval, const siginfo_t* info);

sgx_signal_dispatcher_t *g_signal_dispatch = NULL;
static sgx_spinlock_t g_spin_lock;

inline static unsigned int hash_func(void *p)
{
    key_value_t *kv = p;
    return (unsigned int)kv->signum;
}

inline static int eq_func(void *p, void *q)
{
    key_value_t *kv1 = p;
    key_value_t *kv2 = q;
    return kv1->signum == kv2->signum;
}

int signal_dispatcher_init(void)
{
    g_signal_dispatch = (sgx_signal_dispatcher_t *)malloc(sizeof(sgx_signal_dispatcher_t));
    if (g_signal_dispatch == NULL) {
        return -1;
    }
    g_signal_dispatch->signal_to_eid_set = htab_create();
    if (g_signal_dispatch->signal_to_eid_set == NULL) {
        free(g_signal_dispatch);
        g_signal_dispatch = NULL;
        return -1;
    }

    pthread_mutex_init(&g_signal_dispatch->lock, NULL);
    return 0;
}

int signal_dispatcher_uninit(void)
{
    if (g_signal_dispatch == NULL) {
        return -1;
    }
    pthread_mutex_lock(&g_signal_dispatch->lock);
    htab_delete(g_signal_dispatch->signal_to_eid_set);
    pthread_mutex_unlock(&g_signal_dispatch->lock);
    free(g_signal_dispatch);
    g_signal_dispatch = NULL;
    return 0;
}

void signal_dispatcher_instance_init(void) {
    sgx_spin_lock(&g_spin_lock);
    if (g_signal_dispatch == NULL) {
        if (signal_dispatcher_init() < 0) {
            sgx_spin_unlock(&g_spin_lock);
            return;
        }
    }
    sgx_spin_unlock(&g_spin_lock);
}

static int get_eid_for_signal(int signum, unsigned long long *eid)
{
    void **entry = NULL;
    key_value_t *kv = NULL;
    key_value_t keyvalue = {signum, 0};

    signal_dispatcher_instance_init();

    pthread_mutex_lock(&g_signal_dispatch->lock);
    entry = htab_find_slot(g_signal_dispatch->signal_to_eid_set, &keyvalue, 0, hash_func, eq_func);
    if (!entry) {
        pthread_mutex_unlock(&g_signal_dispatch->lock);
        return -1;
    }
     //exist
    if (*entry) {
        kv = (key_value_t *)*entry;
        if (eid) {
            *eid = kv->enclave_id;
        }
    }
    pthread_mutex_unlock(&g_signal_dispatch->lock);
    return 0;
}

unsigned long long signal_register(int signum, unsigned long long enclave_id)
{
    sigset_t mask = {0};
    sigset_t old_mask = {0};
    void **entry;
    key_value_t *kv = NULL;
    key_value_t keyvalue = {signum, 0};
    unsigned long long old_eid = 0;

    signal_dispatcher_instance_init();
    // Block all signals when registering a signal handler to avoid deadlock.
    sigfillset(&mask);
    sigprocmask(SIG_SETMASK, &mask, &old_mask);
    pthread_mutex_lock(&g_signal_dispatch->lock);
    entry = htab_find_slot(g_signal_dispatch->signal_to_eid_set, &keyvalue, 1, hash_func, eq_func);
    if (!entry) {
        pthread_mutex_unlock(&g_signal_dispatch->lock);
        sigprocmask(SIG_SETMASK, &old_mask, NULL);
        return 0;
    }
    //exist
    if (*entry) {
        kv = (key_value_t *)(*entry);
        old_eid = kv->enclave_id;
    } else {
        kv = (key_value_t *)malloc(sizeof(key_value_t));
        if (kv) {
            kv->signum = signum;
            kv->enclave_id = enclave_id;
            *entry = kv;
        }
    }

    pthread_mutex_unlock(&g_signal_dispatch->lock);
    sigprocmask(SIG_SETMASK, &old_mask,NULL);
    return old_eid;
}

int deregister_all_signals_for_eid(unsigned long long eid)
{
    sigset_t mask = {0};
    sigset_t old_mask = {0};
    key_value_t *kv = NULL;
    size_t i = 0;

    signal_dispatcher_instance_init();

    sigfillset(&mask);
    sigprocmask(SIG_SETMASK, &mask, &old_mask);
    // If this enclave has registered any signals, deregister them and set the
    // signal handler to the default one.
    pthread_mutex_lock(&g_signal_dispatch->lock);
     for (i = g_signal_dispatch->signal_to_eid_set->size; i > 0; i--) {
         if (g_signal_dispatch->signal_to_eid_set->entries[i - 1]) {
            kv = (key_value_t*)g_signal_dispatch->signal_to_eid_set->entries[i - 1];
            if (kv->enclave_id == eid) {
                if (signal(kv->signum, SIG_DFL) == SIG_ERR) {
                   //error
                }
                free(g_signal_dispatch->signal_to_eid_set->entries[i]);
                g_signal_dispatch->signal_to_eid_set->entries[i] = NULL;
            }
        }
    }
    pthread_mutex_unlock(&g_signal_dispatch->lock);
    sigprocmask(SIG_SETMASK, &old_mask, NULL);
    return 0;
}

static int handle_signal(int signum, const siginfo_t *info, __attribute__ ((unused))const void *context)
{
    int ret = 0;
    unsigned int result = 0;
    unsigned long long eid = 0;

    ret = get_eid_for_signal(signum, &eid);
    if (ret < 0) {
        return -1;
    }
    result = t_signal_handler_ecall(eid, &ret, info);
    if (result != 0) {
        return -1;
    }
    return ret;
}

void handle_signal_entry(int signum, siginfo_t *info, void * ucontext) {
    if (info == NULL) {
        return;
    }
    handle_signal(signum, info, ucontext);
}

int u_sigaction_ocall(int *error,
                      int signum,
                      const void *act,
                      void *old_act,
                      unsigned long long  enclave_id)
{
    struct sigaction *e_act = (struct sigaction *)act;
    int ret = 0;
    struct sigaction newact = {0};
    struct sigaction oldact = {0};

    if (signum <= 0 || signum >= NSIG || act == NULL) {
        if (error) {
            *error = EINVAL;
        }
        return -1;
    }
    if (e_act->sa_sigaction == 0) {
        signal_register(signum, enclave_id);
        newact.sa_sigaction = handle_signal_entry;
       // Set the flag so that sa_sigaction is registered as the signal handler
       // instead of sa_handler.
       newact.sa_flags = e_act->sa_flags ;
       newact.sa_flags |= SA_SIGINFO;
       newact.sa_mask = e_act->sa_mask;
        ret = sigaction(signum, &newact, &oldact);
    } else {
        ret = sigaction(signum, act, old_act);
    }
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_sigprocmask_ocall(int *error,
                        int signum,
                        const sigset_t *set,
                        sigset_t * oldset)
{
    int ret = sigprocmask(signum, set, oldset);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_raise_ocall(int signum)
{
    return raise(signum);
}

void u_signal_clear_ocall(unsigned long long  enclave_id)
{
    deregister_all_signals_for_eid(enclave_id);
}
