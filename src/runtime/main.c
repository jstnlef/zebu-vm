#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>

extern void* vm;
extern void mu_main(char*, int, char**);

int main(int argc, char** argv) {
    char* serialize_vm = (char*) &vm;
    mu_main(serialize_vm, argc, argv);
}
