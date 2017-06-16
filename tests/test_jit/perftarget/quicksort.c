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

#include <stdint.h>
#include "quicksort.h"
static inline void swap(int64_t* arr, int64_t i, int64_t j) {
    int64_t t;
    t = arr[i];
    arr[i] = arr[j];
    arr[j] = t;
}

static inline int64_t partition(int64_t* arr, int64_t idx_low, int64_t idx_high) {
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
