// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#ifdef __linux__
// RTLD_DEFAULT is not defined in POSIX. Linux gcc does not define it unless
// _GNU_SOURCE is also defined.
#define _GNU_SOURCE
#endif // __linux__

#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <pthread.h>
#include <assert.h>

int tls_initialized = 0;

static pthread_key_t mu_tls;

void set_thread_local(void* thread) {
    if(tls_initialized == 0){
        tls_initialized = 1;
        int result = pthread_key_create(&mu_tls, NULL);
        if(result != 0){
            printf("Set_Thread_Local(): PThread key create failed with error code = %d\n", result);
            assert(0);
        }
    }
    int result = pthread_setspecific(mu_tls, thread);
    if(result != 0){
        printf("Set_Thread_Local(): PThread set specific failed with error code = %d\n", result);
        assert(0);
    }
}

void* muentry_get_thread_local() {
    if(tls_initialized == 0){
        printf("Get_Thread_Local(): PThread key MUST be initialized before first use!!\n");
        assert(0);
    }
    void * result = pthread_getspecific(mu_tls);
    if(result == NULL){
        printf("Get_Thread_Local(): NO pthread key found for current thread!!\n");
        assert(0);
    }

    return result;
}

int32_t mu_retval;

void muentry_set_retval(int32_t x) {
    mu_retval = x;
}

int32_t muentry_get_retval() {
    return mu_retval;
}

int32_t c_check_result() {
    return mu_retval;
}

char * alloc_mem(size_t size){
    return (char *) malloc(size);
}
