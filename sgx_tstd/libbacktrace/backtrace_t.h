#ifndef BACKTRACE_BACKTRACE_T_H
#define BACKTRACE_BACKTRACE_T_H

#include <stdint.h>
#include <wchar.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

uint32_t u_open_ocall(int* retval, int* error, const char* pathname, int flags);
uint32_t u_close_ocall(int* retval, int* error, int fd);
uint32_t u_fcntl_arg1_ocall(int* retval, int* error, int fd, int cmd, int arg);
uint32_t u_mmap_ocall(void** retval, int* error, void* start, size_t length, int prot,
                                int flags, int fd, int64_t offset);
uint32_t u_munmap_ocall(int* retval, int* error, void* start, size_t length);

#ifdef __cplusplus
}
#endif /* __cplusplus */

#endif