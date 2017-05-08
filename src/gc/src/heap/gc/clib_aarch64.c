#include <inttypes.h>
#include <stdlib.h>
#include <string.h>

void* malloc_zero(size_t size) {
    void* ret = malloc(size);
    memset(ret, 0, size);
    return ret;
}

//uintptr_t immmix_get_stack_ptr()
__asm__(".text\n"
        ".global immmix_get_stack_ptr\n"
        ".type immmix_get_stack_ptr,@function\n"
        ".balign 16\n"
        "immmix_get_stack_ptr:\n"

        "MOV X0, SP\n"
        "RET\n"

        ".size immmix_get_stack_ptr, .-immmix_get_stack_ptr\n"
);

int get_registers_count() {
    return 31;
}

// uintptr_t* get_registers ()
__asm__(".text\n"
        ".global get_registers\n"
        ".type get_registers,@function\n"
        ".balign 16\n"
        "get_registers:\n"

        // Start frame
        "STP X29, X30, [SP, #-16]!\n"
        "MOV X29, SP\n"

        // push registers onto the stack
        "STP X27, X28,[SP, #-16]!\n"
        "STP X25, X26,[SP, #-16]!\n"
        "STP X23, X24,[SP, #-16]!\n"
        "STP X21, X22,[SP, #-16]!\n"
        "STP X19, X20,[SP, #-16]!\n"
        "STP X17, X18,[SP, #-16]!\n"
        "STP X15, X16,[SP, #-16]!\n"
        "STP X13, X14,[SP, #-16]!\n"
        "STP X11, X12,[SP, #-16]!\n"
        "STP X9, X10,[SP, #-16]!\n"
        "STP X7, X8,[SP, #-16]!\n"
        "STP X5, X6,[SP, #-16]!\n"
        "STP X3, X4,[SP, #-16]!\n"
        "STP X1, X2,[SP, #-16]!\n"

        "STP XZR, X0,[SP, #-16]!\n"

        // sizeof(uintptr_t) * 31
        "MOV X0, #244\n" // 244 bytes to allocate
        "BL malloc\n"
        // Now X0 contains the value returned by malloc
        //ret[0] = x0; (use X2 and X3 as temporaries)
        "MOV X1, X0\n" // Make a copy of X0, that can be freely modified (X0 will be returned)
        "LDP XZR, X2, [SP],#16\n" // X2 = original value of X0
        "STR X2, [X1],#8\n" // X1[0] = original value of X0

        // Pop the top two registers from the stack, and store them in X1, and increment x1
        // (do this 15 times for each pair of register (ignoring X0, which was popped above)
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"
        "LDP X2, X3, [SP],#16\n"
        "STP X2, X3, [X1],#16\n"

        // X0 contains the value returned from malloc
        // The frame pointer and link register were popped above, so they have now been restored
        "RET X30\n"
        ".size get_registers, .-get_registers\n"
);


__thread uintptr_t low_water_mark;

// void set_low_water_mark()
__asm__(".text\n"
        ".global set_low_water_mark\n"
        ".type set_low_water_mark,@function\n"
        ".balign 16\n"
        "set_low_water_mark:\n"

        // Save stack pointer
        "MOV X0, SP\n"

        // Store frame record
        "STP X29, X30, [SP, #-16]!\n"
        "MOV X29, SP\n"

        // Call the c function that actually does the set (taking X0 as the argument)
        "BL set_low_water_mark_internal\n"

        // Restore frame record
        "LDP X29, X30, [SP], #16\n"
        "RET\n"

        ".size set_low_water_mark, .-set_low_water_mark\n"
);

// Internal use only (used by set_low_water_mark)
void set_low_water_mark_internal (uintptr_t sp) {
    low_water_mark = sp;
}

uintptr_t get_low_water_mark() {
    return low_water_mark;
}
