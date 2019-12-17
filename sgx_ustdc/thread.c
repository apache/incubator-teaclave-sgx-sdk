#include <pthread.h>
#include <sched.h>
#include <time.h>
#include <errno.h>
#include <stdlib.h>
#include <string.h>

#define THREAD_PARAM_SEALED_SIZE   576
struct ThreadParam {
    unsigned char main[THREAD_PARAM_SEALED_SIZE];
    unsigned long long eid;
};

unsigned int t_thread_main(unsigned long long eid, void **retval, void* arg, int len);

static void *thread_start(void *param)
{
    void* retval = NULL;
    struct ThreadParam *tp = (struct ThreadParam *)param;   
    if (tp) {
        unsigned long long eid = tp->eid;
        unsigned int ret = t_thread_main(eid, &retval, tp, sizeof(struct ThreadParam));
        if (ret != 0) {
           retval = (void *)((size_t)ret);
        }
        free(tp);
    }
    return retval;
}

int u_pthread_create_ocall(pthread_t *thread,
                           const pthread_attr_t *attr,
                           void *(*start_routine) (void *),
                           void *arg,
                           int len) 
 {
    if (arg == NULL || len != sizeof(struct ThreadParam)) {
        return EINVAL;
    }
    struct ThreadParam *tmp = (struct ThreadParam *)malloc(len);
    if (tmp == NULL) {
        return ENOMEM;
    }
    memcpy((void*)tmp, arg, len);

    int ret = pthread_create(thread, attr, thread_start, tmp);
    if (ret != 0) {
        if (tmp) free(tmp);
    }
    return ret;  
}

int u_pthread_join_ocall(pthread_t thread, void **retval) 
{
    return pthread_join(thread, retval);      
}

int u_pthread_detach_ocall(pthread_t thread)
{
    return pthread_detach(thread);              
}

int  u_sched_yield_ocall(int * error) 
{
    int ret = sched_yield();
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;
}

int u_nanosleep_ocall(int * error, const struct timespec *req, struct timespec *rem) 
{
    int ret = nanosleep(req, rem);
    if (error) {
        *error = ret == -1 ? errno : 0;
    }
    return ret;          
}


