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

#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <time.h>
#include <assert.h>
extern void* vm;
extern void* RODAL_END;
extern void mu_main(void*, void*, int, char**);
extern void rodal_init_deallocate(void);
extern void rodal_free(void*);
extern void* rodal_realloc(void*, size_t);

extern uint32_t mu_retval;

int main(int argc, char** argv) {
    rodal_init_deallocate();
    mu_main(&RODAL_END, &vm, argc, argv);
    return (int)mu_retval;
}

void free(void* ptr) { return rodal_free(ptr); };
void* realloc(void* ptr, size_t s) { return rodal_realloc(ptr, s); };