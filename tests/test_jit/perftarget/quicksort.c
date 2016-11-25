#include <stdint.h>
#include "quicksort.h"
void swap(int64_t* arr, int64_t i, int64_t j) {
    int64_t t;
    t = arr[i];
    arr[i] = arr[j];
    arr[j] = t;
}

int64_t partition(int64_t* arr, int64_t idx_low, int64_t idx_high) {
    int64_t pivot, i, j;
    pivot = arr[idx_high];
    i = idx_low;
    for (j = idx_low; j < idx_high; j ++) {
        if (arr[j] < pivot) {
            swap(arr, i, j);
            i += 1;
        }
    }
    swap(arr, i, idx_high);
    return i;
}

void quicksort(int64_t* arr, int64_t start, int64_t end) {
    int64_t p;
    if (start < end) {
        p = partition(arr, start, end);
        quicksort(arr, start, p - 1);
        quicksort(arr, p + 1, end);
    }
}
