#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <dlfcn.h>

__thread void* mu_tls;

void set_thread_local(void* thread) {
    printf("setting mu_tls to %p\n", thread);
    mu_tls = thread;
}

void* get_thread_local() {
    printf("getting mu_tls as %p\n", mu_tls);
    return mu_tls;
}

void* resolve_symbol(const char* sym) {
    printf("%s\n", sym);
    return dlsym(RTLD_DEFAULT, sym);
}
