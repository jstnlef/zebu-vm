#include <stdint.h>
#include <stdlib.h>

__thread void* mu_tls;

void* init_thread_local(void** local) {
    mu_tls = *local;

    return &mu_tls;
}
