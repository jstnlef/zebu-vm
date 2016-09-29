#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <dlfcn.h>
#include <pthread.h>

__thread void* mu_tls;

void set_thread_local(void* thread) {
    printf("Thread%p: setting mu_tls to %p\n", pthread_self(), thread);
    mu_tls = thread;
}

void* muentry_get_thread_local() {
    printf("Thread%p: getting mu_tls as %p\n", pthread_self(), mu_tls);
    return mu_tls;
}

void* resolve_symbol(const char* sym) {
    printf("%s\n", sym);
    return dlsym(RTLD_DEFAULT, sym);
}
