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
