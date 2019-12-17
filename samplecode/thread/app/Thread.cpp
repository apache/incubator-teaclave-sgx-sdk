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

#include <thread>
#include <stdio.h>
using namespace std;

#include "App.h"
#include "Enclave_u.h"

void data_producer(void)
{
    sgx_status_t ret = SGX_ERROR_UNEXPECTED;
    ret = ecall_producer(global_eid);
    if (ret != SGX_SUCCESS)
        abort();
}

void data_consumer(void)
{
    sgx_status_t ret = SGX_ERROR_UNEXPECTED;
    ret = ecall_consumer(global_eid);
    if (ret != SGX_SUCCESS)
        abort();
}

void data_init(void)
{
    sgx_status_t ret = SGX_ERROR_UNEXPECTED;
    ret = ecall_initialize(global_eid);
    if (ret != SGX_SUCCESS)
        abort();
}

void data_uninit(void)
{
    sgx_status_t ret = SGX_ERROR_UNEXPECTED;
    ret = ecall_uninitialize(global_eid);
    if (ret != SGX_SUCCESS)
        abort();
}

/* ecall_thread_functions:
 *   Invokes thread functions including mutex, condition variable, etc.
 */
void ecall_thread_functions(void)
{
    data_init();

    printf("Info: executing thread synchronization, please wait...  \n");
    /* condition variable */
    thread consumer1(data_consumer);
    thread producer0(data_producer);
    thread producer1(data_producer);
    thread producer2(data_producer);
    thread producer3(data_producer);
    thread consumer2(data_consumer);
    thread consumer3(data_consumer);
    thread consumer4(data_consumer);

    consumer1.join();
    consumer2.join();
    consumer3.join();
    consumer4.join();
    producer0.join();
    producer1.join();
    producer2.join();
    producer3.join();

    printf("Info: thread finish...  \n");

    data_uninit();
}
