#include <stdio.h>
#include <stdint.h>

#include <muapi.h>
#include "cpart.h"

int main() {
    printf("Hello world!\n");

    MuVM *mvm = new_mock_micro_vm("cclientt");

    MuID forty_two = mvm->id_of(mvm, "@forty_two");

    printf("forty_two = %d\n", forty_two);

    MuName name_43 = mvm->name_of(mvm, 43);
    printf("name_43 = '%s'\n", name_43);

    MuName name_43_2 = mvm->name_of(mvm, 43);
    printf("name_43_2 = '%s'\n", name_43_2);

    // TODO: Register a trap handler to see if Rust can really call back.

    free_mock_micro_vm(mvm);

    return 0;
}
