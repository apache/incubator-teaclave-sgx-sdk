#include <stdio.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
// #include <sys/time.h>
#include <time.h>
#include <stdarg.h> // for variable arguments functions
#include <fcntl.h>
#include <stdlib.h>

// At this point we have already definitions needed for ocall interface, so:
#define DO_NOT_REDEFINE_FOR_OCALL
#include "Enclave_t.h"

// For open64 need to define this
#define O_TMPFILE (__O_TMPFILE | O_DIRECTORY)

long int sysconf(int name){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

int open64(const char *filename, int flags, ...){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

off_t lseek64(int fd, off_t offset, int whence){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

int gettimeofday(struct timeval *tv, struct timezone *tz){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

unsigned int sleep(unsigned int seconds){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

void *dlopen(const char *filename, int flag){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return NULL;
}

char *dlerror(void){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return NULL;
}

void *dlsym(void *handle, const char *symbol){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
}

int dlclose(void *handle){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

int utimes(const char *filename, const struct timeval times[2]){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

struct tm *localtime(const time_t *timep){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return NULL;
}

pid_t getpid(void){
  pid_t ret = 0;
  
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  
  return ret;
}

int fsync(int fd){
  int ret = 0;
  
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  
  return ret;
}

time_t time(time_t *t){
  time_t ret;
  
  memset(&ret, 0x0, sizeof(ret));
  
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  
  return ret;
}

int close(int fd){
  int ret = 0;
  
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  
  return ret;
}

int access(const char *pathname, int mode){
  int ret = 0;
  
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  
  return ret;
}

char *getcwd(char *buf, size_t size){
  char* ret = NULL;

  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);

  return ret;
}

int sgx_lstat( const char* path, struct stat *buf ) {
  int ret = 0;
  
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  
  return ret;
}

int sgx_stat(const char *path, struct stat *buf){
  int ret = 0;
  
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  
  return ret;
}

int sgx_fstat(int fd, struct stat *buf){
  int ret = 0;
  
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  
  return ret;
}

int sgx_ftruncate(int fd, off_t length){
  int ret = 0;

  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  
  return ret;
}

int fcntl(int fd, int cmd, ... /* arg */ ){
  int ret = 0;

  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);

  return ret;
}

ssize_t read(int fd, void *buf, size_t count){
  ssize_t ret = 0;

  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);

  return ret;
}

ssize_t write(int fd, const void *buf, size_t count){
  ssize_t ret = 0;

  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);

  return ret;
}

int fchmod(int fd, mode_t mode){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

int unlink(const char *pathname){
  int ret = 0;

  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  
  return ret;
}

int mkdir(const char *pathname, mode_t mode) {
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

int rmdir(const char *pathname){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

int fchown(int fd, uid_t owner, gid_t group){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

uid_t geteuid(void){
  uid_t ret = 0;

  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);

  return ret;
}

char* getenv(const char *name){
  char* ret = NULL;

  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);

  return ret;
}

void *mmap64(void *addr, size_t len, int prot, int flags, int fildes, off_t off){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
}

int munmap(void *addr, size_t length){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}

void *mremap(void *old_address, size_t old_size, size_t new_size, int flags, ... /* void *new_address */){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
}

ssize_t readlink(const char *path, char *buf, size_t bufsiz){
  char error_msg[256];
  snprintf(error_msg, sizeof(error_msg), "%s%s", "Error: no ocall implementation for ", __func__);
  ocall_print_error(error_msg);
  return 0;
}


int callback(void *NotUsed, int argc, char **argv, char **azColName){
    int i;
    for (i=0; i < argc; i++){
        ocall_print_string(azColName[i]);
        ocall_print_string(" = ");
        ocall_print_string(argv[i]);
        ocall_print_string("\n"); 
    }
    ocall_print_string("\n");
    return 0;
}
