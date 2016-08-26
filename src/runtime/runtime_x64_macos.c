#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <dlfcn.h>

__thread void* mu_tls;

void* init_thread_local(void* thread) {
    mu_tls = thread;

    return &mu_tls;
}

void* resolve_symbol(const char* sym) {
    printf("%s\n", sym);
    return dlsym(RTLD_DEFAULT, sym);
}
