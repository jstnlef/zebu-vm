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
#include <dlfcn.h>
#include <pthread.h>

uint32_t mu_retval;

__thread void* mu_tls;

void muentry_set_retval(uint32_t x) {
    mu_retval = x;
}

void set_thread_local(void* thread) {
    // printf("Thread%p: setting mu_tls to %p\n", (void*) pthread_self(), thread);
    mu_tls = thread;
}

void* muentry_get_thread_local() {
//    printf("Thread%p: getting mu_tls as %p\n", (void*) pthread_self(), mu_tls);
    return mu_tls;
}

void* resolve_symbol(const char* sym) {
    // printf("%s\n", sym);
    return dlsym(RTLD_DEFAULT, sym);
}

int32_t mu_retval;

void muentry_set_retval(int32_t x) {
    mu_retval = x;
}

int32_t c_check_result() {
    return mu_retval;
}

char * alloc_mem(size_t size){
    return (char *) malloc(size);
}
