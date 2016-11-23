
#include <stdint.h>

uint64_t arraysum(int64_t* arr, uint64_t sz) {
    int64_t sum = 0;
    uint64_t i;
    for(i = 0; i < sz; i ++) {
        sum += arr[i];
    }
    return sum;
}
