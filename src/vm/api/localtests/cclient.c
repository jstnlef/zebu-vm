#include <stdio.h>
#include <stdint.h>

#include <muapi.h>
#include "cpart.h"

int main() {
    printf("Hello world!\n");

    MuVM *mvm = new_mock_micro_vm("cclientt");

    free_mock_micro_vm(mvm);

    return 0;
}
