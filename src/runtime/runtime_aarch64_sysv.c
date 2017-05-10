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

/*
 *         .type   mu_tls,@object          // @mu_tls
        .section        .tbss,"awT",@nobits
        .globl  mu_tls
        .p2align        3
mu_tls:
        .xword  0
        .size   mu_tls, 8

 * */
__thread void* mu_tls;



void set_thread_local(void* thread) {
    // printf("Thread%p: setting mu_tls to %p\n", (void*) pthread_self(), thread);
    //MRS X8, TPIDR_EL0
    //ADD X8, X8, :tprel_hi12:mu_tls
    //ADD X8, X8, :tprel_lo12_nc:mu_tls
    //STR X0, [X8]


    mu_tls = thread;
}

void* muentry_get_thread_local() {
//    printf("Thread%p: getting mu_tls as %p\n", (void*) pthread_self(), mu_tls);
    /*
 * //MRS X8, TPIDR_EL0
    //ADD X8, X8, :tprel_hi12:mu_tls
    //ADD X8, X8, :tprel_lo12_nc:mu_tls
    //LDR X0, [X8]
     *
     * */
    return mu_tls;
}

void* resolve_symbol(const char* sym) {
    // MOV X1, X0
    // MOV X0, XZR
    // B dlsym
    // printf("%s\n", sym);
    return dlsym(RTLD_DEFAULT, sym);
}