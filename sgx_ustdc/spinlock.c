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

#include "spinlock.h"

static inline void _mm_pause(void) __attribute__((always_inline));
static inline int _InterlockedExchange(int volatile *dst, int val) __attribute__((always_inline));

static inline void _mm_pause(void)  /* definition requires -ffreestanding */
{
    __asm __volatile(
        "pause"
    );
}

static inline int _InterlockedExchange(int volatile *dst, int val)
{
    int res;
    __asm __volatile(
        "lock xchg %2, %1;"
        "mov %2, %0"
        : "=m" (res)
        : "m" (*dst),
        "r" (val) 
        : "memory"
    );
    return (res);
   
}

uint32_t sgx_spin_lock(sgx_spinlock_t *lock)
{
    while(_InterlockedExchange((volatile int *)lock, 1) != 0) {
        while (*lock) {
            /* tell cpu we are spinning */
            _mm_pause();
        } 
    }
    return (0);
}

uint32_t sgx_spin_unlock(sgx_spinlock_t *lock)
{
    *lock = 0;
    return (0);
}

