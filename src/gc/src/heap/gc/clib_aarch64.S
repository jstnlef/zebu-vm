# Copyright 2017 The Australian National University
# 
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
# 
#     http://www.apache.org/licenses/LICENSE-2.0
# 
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
    RET
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
	push_pair X0, XZR

    # sizeof(uintptr_t) * 31
	MOV X0, #248 // 248 bytes to allocate
	BL malloc // X0 = malloc(244)

	MOV X1, X0 // Make a copy of X0, that can be freely modified (X0 will be returned)

    pop_pair XZR, X2
	STR X2, [X1],#8 // X1[0] = original value of X0

    # Pop the top two registers from the stack, and store them in X1, and increment X1 and SP
    # (do this 15 times for each pair of register (ignoring X0, which was popped above))
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

	pop_pair X29, X30 // Restore the FP and LR and save them
	STP X29, X30, [X1]

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

