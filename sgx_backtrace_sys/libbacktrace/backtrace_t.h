#ifndef BACKTRACE_BACKTRACE_T_H
#define BACKTRACE_BACKTRACE_T_H

#include <stdint.h>
#include <wchar.h>
#include <stddef.h>
#include <sys/stat.h>

#ifdef __cplusplus
extern "C" {
#endif

uint32_t u_open_ocall(int* retval, int* error, const char* pathname, int flags);
uint32_t u_close_ocall(int* retval, int* error, int fd);
uint32_t u_fcntl_arg1_ocall(int* retval, int* error, int fd, int cmd, int arg);
uint32_t u_mmap_ocall(void** retval, int* error, void* start, size_t length, int prot, int flags, int fd, int64_t offset);
uint32_t u_munmap_ocall(int* retval, int* error, void* start, size_t length);

uint32_t u_fstat_ocall(int* retval, int* error, int fd, struct stat* buf);
uint32_t u_lstat_ocall(int* retval, int* error, const char* path, struct stat* buf);

uint32_t u_read_ocall(size_t* retval, int* error, int fd, void* buf, size_t count);
uint32_t u_lseek_ocall(uint64_t* retval, int* error, int fd, int64_t offset, int whence);

#ifdef __cplusplus
}
#endif /* __cplusplus */

#endif