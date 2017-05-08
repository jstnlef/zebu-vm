#include <stdint.h>

uint64_t fib(uint64_t n) {
    if(n <= 1)
        return n;
    return fib(n - 2) + fib(n - 1);
}