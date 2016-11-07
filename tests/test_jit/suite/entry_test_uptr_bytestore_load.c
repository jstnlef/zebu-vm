#include <stdint.h>

extern uint32_t test_fnc(uint32_t* pi32);

int entry() {
    uint32_t i;
    return test_fnc(&i);
}
