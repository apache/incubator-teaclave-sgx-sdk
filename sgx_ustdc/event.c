
#include <sys/types.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <sys/syscall.h>
#include <pthread.h>
#include <stdbool.h>
#include <string.h>
#include "spinlock.h"
typedef void *se_handle_t;
typedef void *tcs_handle_t;

#define FUTEX_WAIT 0
#define FUTEX_WAKE 1
#define FUTEX_PRIVATE_FLAG 128

#define SE_MUTEX_SUCCESS    0x0
#define SE_MUTEX_INVALID    0x1
#define SE_MUTEX_ERROR_WAKE 0x2
#define SE_MUTEX_ERROR_WAIT 0x3
#define SE_MUTEX_ERROR_TIMEOUT 0x4

struct list_head {
    struct list_head *next, *prev;
};

static inline void
INIT_LIST_HEAD(struct list_head *list)
{
    list->next = list->prev = list;
}

static inline void
__list_add(struct list_head *entry,
                struct list_head *prev, struct list_head *next)
{
    next->prev = entry;
    entry->next = next;
    entry->prev = prev;
    prev->next = entry;
}

static inline void
list_add(struct list_head *entry, struct list_head *head)
{
    __list_add(entry, head, head->next);
}


static inline void
__list_del(struct list_head *prev, struct list_head *next)
{
    next->prev = prev;
    prev->next = next;
}

static inline void
list_del(struct list_head *entry)
{
    __list_del(entry->prev, entry->next);
}

static inline void
list_del_init(struct list_head *entry)
{
    __list_del(entry->prev, entry->next);
    INIT_LIST_HEAD(entry);
}

static inline bool
list_empty(struct list_head *head)
{
    return head->next == head;
}

#ifndef container_of
#define container_of(ptr, type, member) \
    (type *)((char *)(ptr) - (char *) &((type *)0)->member)
#endif

#define list_entry(ptr, type, member) \
    container_of(ptr, type, member)

#define list_first_entry(ptr, type, member) \
    list_entry((ptr)->next, type, member)

#define __container_of(ptr, sample, member)				\
    (void *)container_of((ptr), typeof(*(sample)), member)

#define list_for_each_entry(pos, head, member)				\
    for (pos = __container_of((head)->next, pos, member);		\
	 &pos->member != (head);					\
	 pos = __container_of(pos->member.next, pos, member))

typedef struct _sgx_tcs_info_cache_t {
    pthread_mutex_t lock;
    struct list_head tcs_cache_list;
} sgx_tcs_info_cache_t;

typedef struct _sgx_tcs_info_t {
    struct list_head entry;
    se_handle_t se_event;
    tcs_handle_t tcs;
} sgx_tcs_info_t;

sgx_tcs_info_cache_t *SgxTcsInfoCache = NULL;
static sgx_spinlock_t g_spin_lock;

se_handle_t se_event_init(void)
{
    return calloc(1, sizeof(int)); 
}

void se_event_destroy(se_handle_t se_event)
{
    if (se_event != NULL)
        free(se_event); 
}

int se_event_wait(se_handle_t se_event)
{
    if (se_event == NULL)
        return EINVAL;

    if (__sync_fetch_and_add((int*)se_event, -1) == 0)
        syscall(__NR_futex, se_event, FUTEX_WAIT, -1, NULL, NULL, 0);

    return 0;
}

int se_event_wait_timeout(se_handle_t se_event, const struct timespec *timeout)
{
    long ret = -1;

    if (se_event == NULL) {
        return EINVAL;
    }

    if (timeout == NULL) {
        return se_event_wait(se_event);
    }
    
    if (__sync_fetch_and_add((int *)se_event, -1) == 0) {
        
        ret = syscall(__NR_futex, se_event, FUTEX_WAIT, -1, timeout, 0, 0);
        if (ret < 0) {
            if (errno == ETIMEDOUT) {
                //If the futex is exit with timeout (se_event still equal to ' -1'), the se_event value need reset to 0.
                __sync_val_compare_and_swap((int*)se_event, -1, 0);  
                return -1;
            }
        }
    }
    return 0;
}

int se_event_wake(se_handle_t se_event)
{
    if (se_event == NULL)
        return EINVAL;

    if (__sync_fetch_and_add((int*)se_event, 1) != 0) {
       syscall(__NR_futex, se_event, FUTEX_WAKE, 1, NULL, NULL, 0);  
    }
    
    return 0;
}

sgx_tcs_info_cache_t *sgx_tcs_cache_init(void) 
{   
    sgx_tcs_info_cache_t *cache = (sgx_tcs_info_cache_t *)malloc(sizeof(sgx_tcs_info_cache_t));
    if (cache) {
        pthread_mutex_t mp = PTHREAD_MUTEX_INITIALIZER;
        memcpy (&cache->lock, &mp, sizeof(mp));
        pthread_mutex_init(&cache->lock, NULL);
        INIT_LIST_HEAD(&cache->tcs_cache_list);
    }
    return cache;
}

void sgx_tcs_cache_destory(sgx_tcs_info_cache_t **cache)
{   
    sgx_tcs_info_t *p = NULL;

    if (cache == NULL || *cache == NULL) {
        return;
    }
    pthread_mutex_lock(&(*cache)->lock);
    while (!p && !list_empty(&(*cache)->tcs_cache_list)) {

		p = list_first_entry(&(*cache)->tcs_cache_list,
					sgx_tcs_info_t, entry);
        if (p) {
            
            se_event_destroy(p->se_event);
            free(p);
            list_del(&p->entry);
        }
	}
    pthread_mutex_unlock(&(*cache)->lock);
    pthread_mutex_destroy(&(*cache)->lock);
    free(*cache);
    *cache = NULL;
}

se_handle_t sgx_tcs_event_get(sgx_tcs_info_cache_t *cache, const tcs_handle_t tcs) 
{
    sgx_tcs_info_t *p = NULL;
    se_handle_t se_event = NULL;

    if (cache == NULL) {
        return NULL;
    }       
    pthread_mutex_lock(&cache->lock);
    list_for_each_entry(p, &cache->tcs_cache_list, entry) {

        if (p->tcs == tcs) {
            se_event = p->se_event;        
            pthread_mutex_unlock(&cache->lock);
            return se_event;
        }
    }
    
    p = (sgx_tcs_info_t*)malloc(sizeof(sgx_tcs_info_t));
    if (p == NULL) {
        pthread_mutex_unlock(&cache->lock);
        return NULL;
    }

    p->tcs = tcs;
    se_event = p->se_event = se_event_init();
   
    list_add(&p->entry, &cache->tcs_cache_list);
    pthread_mutex_unlock(&cache->lock);
    return se_event;
}

se_handle_t get_tcs_event(const tcs_handle_t tcs)
{   
    se_handle_t se_handle;

    sgx_spin_lock(&g_spin_lock);
    if (SgxTcsInfoCache == NULL) {

        SgxTcsInfoCache = sgx_tcs_cache_init();
    }
    sgx_spin_unlock(&g_spin_lock);
    se_handle = sgx_tcs_event_get(SgxTcsInfoCache, tcs);
   
    return se_handle;
}

int u_thread_set_event_ocall(int *error, const tcs_handle_t tcs)
{
    se_handle_t se_event = NULL;

    if (error)
        *error = EINVAL;

    if (tcs == NULL)
        return -1;

    se_event = get_tcs_event(tcs);
    if (se_event == NULL) {
        return -1;
    }

    int ret = se_event_wake(se_event); 
    if (ret != 0) {
        if (error) {
            *error = errno;   
        }
        return -1;
    }

    if (error)
        *error = 0;

    return 0;
}

int u_thread_wait_event_ocall(int *error, const tcs_handle_t tcs, const struct timespec *timeout)
{
    se_handle_t se_event = NULL;

    if (error)
        *error = EINVAL;

    if (tcs == NULL) {
        return -1;
    }
      
    se_event = get_tcs_event(tcs);
    if (se_event == NULL) {
       return -1;
    }

    int ret = 0;
    if (timeout == NULL) {
        ret = se_event_wait(se_event);
    } else {
        ret = se_event_wait_timeout(se_event, timeout);
    }
    if (ret != 0) {
        if (error) {
            *error = errno;    
        }    
        return -1;
    }

    if (error)
        *error = 0;

    return 0;
}

int u_thread_set_multiple_events_ocall(int *error, const tcs_handle_t *tcss, int total)
{
    int i = 0;
    se_handle_t se_event = NULL;

    if (error) {
        *error = EINVAL;
    }
        
    for (i = 0; i < total; i ++) {

        se_event = get_tcs_event(tcss[i]);
        if (se_event == NULL) {
           return -1;
        }

        int ret = se_event_wake(se_event);
        if (ret != 0) {
            if (error) {
                *error = errno;  
            }   
            return -1;
        }
    }

    if (error)
        *error = 0;

    return 0;
}

int u_thread_setwait_events_ocall(int *error,
                                  const tcs_handle_t waiter_tcs,
                                  const tcs_handle_t self_tcs,
                                  const struct timespec *timeout)
{
    int result = u_thread_set_event_ocall(error, waiter_tcs);
    if (result < 0) {
        return result;
    }

    return u_thread_wait_event_ocall(error, self_tcs, timeout);
}                                      
