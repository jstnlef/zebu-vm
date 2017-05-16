.arch armv8-a

#include "../../../../runtime/asm_common_aarch64.S.inc"

//__thread uintptr_t low_water_mark
    .type low_water_mark,@object
    .section tbss,"awT",@nobits
    .globl low_water_mark
    .balign 8
low_water_mark:
    .xword 0
    .size low_water_mark, 8

#malloc_zero(size_t size) -> void*
#               X0            X0
begin_func malloc_zero
    MOV X1, X0 // X1 = size
    MOV X0, #1
    B calloc // tail call calloc(1, size)
end_func malloc_zero

#immmix_get_stack_ptr() -> uintptr_t 
#                          X0
begin_func immmix_get_stack_ptr
    MOV X0, SP
    RET
end_func immmix_get_stack_ptr

#get_registers_count() -> int 
#                         W0
begin_func get_registers_count
    MOV X0, 31
end_func get_registers_count

# get_registers() -> uintptr_t*
#                    X0
begin_func get_registers
    push_pair X30, X29
    mov x29, sp

    # push registers onto the stack
	push_pair X28, X27
	push_pair X26, X25
	push_pair X24, X23
	push_pair X22, X21
	push_pair X20, X19
	push_pair X18, X17
	push_pair X16, X15
	push_pair X14, X13
	push_pair X12, X11
	push_pair X10, X9
	push_pair X8, X7
	push_pair X6, X5
	push_pair X4, X3
	push_pair X2, X1
	push_pair XZR, X0

    # sizeof(uintptr_t) * 31
	MOV X0, #244 // 244 bytes to allocate
	BL malloc
        
    # Now X0 contains the value returned by malloc
    # ret[0] = x0; (use X2 and X3 as temporaries)
	MOV X1, X0 // Make a copy of X0, that can be freely modified (X0 will be returned)
	LDP XZR, X2, [SP],#16 // X2 = original value of X0
	STR X2, [X1],#8 // X1[0] = original value of X0

    # Pop the top two registers from the stack, and store them in X1, and increment x1
    # (do this 15 times for each pair of register (ignoring X0, which was popped above)
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16
	STP X2, X3, [X1],#16
	LDP X2, X3, [SP],#16 // These will be the frame pointer and link register
	STP X2, X3, [X1],#16

    # X0 contains the value returned from malloc
    # The frame pointer and link register were popped above, so they have now been restored
	RET X30
end_func get_registers

# set_low_water_mark()
begin_func set_low_water_mark
    // low_water_mark = SP
    MRS x8, TPIDR_EL0
    ADD X8, X8, :tprel_hi12:low_water_mark
    ADD X8, X8, :tprel_lo12_nc:low_water_mark
    MOV SP, X0 // cant store the SP directly
    STR X0, [x8]
    RET
end_func set_low_water_mark

# get_low_water_mark() -> uintptr_t
#                         X0
begin_func get_low_water_mark
    MRS x8, TPIDR_EL0
    ADD X8, X8, :tprel_hi12:low_water_mark
    ADD X8, X8, :tprel_lo12_nc:low_water_mark
    LDR X0, [x8]
    RET
end_func get_low_water_mark

