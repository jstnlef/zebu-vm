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

extern void* vm;
extern void mu_main(char*, int, char**);
extern uint32_t mu_retval;

int main(int argc, char** argv) {
    char* serialize_vm = (char*) &vm;
    mu_main(serialize_vm, argc, argv);
    return (int) mu_retval;
}
