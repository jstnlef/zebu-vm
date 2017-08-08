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

#include "asm_common_aarch64.S.inc"


# swap_stack_to(new_sp: Address, entry: Address, old_sp_loc: Address)
#               X0               X1              X2
begin_func swap_to_mu_stack
          # -- on old stack --
          enter_frame

          # push all callee-saved registers
          push_callee_saved

          # *old_sp_loc = SP
          MOV X9, SP
          STR X9, [X2]

          # sp = new_sp
          MOV SP, X0
          # x9 = entry
          MOV X9, X1

          # -- on new stack --
          # arguments (reverse order of thread.rs - runtime_load_args)
          pop_arguments

          # If entry tries to return (IT SHOULDN'T) it will call entry_returned (which will panic!)
          ADR LR, entry_returned

          # branch to entry
          //MOV FP, 0 // End of call frame
          RET X9
end_func swap_to_mu_stack

# The error message to print when entry returns
	.type .Lentry_returned_message, @object
	.section .rodata.str1.1,"aMS",@progbits,1
.Lentry_returned_message:
    .asciz  "ERROR: The entry function returned\n"
    .size   .Lentry_returned_message, 36

# Prints an error mesage and calls exit(1)
begin_func entry_returned
    ADRP x0, .Lentry_returned_message
    ADD x0, x0, :lo12:.Lentry_returned_message // Loads x0 with the adress of .Lentry_returned_message

    MOV X1, #35 // Length of string

	MOV X2, #1 // Number of elements

    ADRP X3, :got:stderr
    LDR X3, [X3, :got_lo12:stderr]
    LDR X3, [X3] // Load X3 with the address of stderr

    BL fwrite // calls fwrite(x0, x1, x2, x3)
    MOV W0, 1
    BL exit // call exit(1)
end_func entry_returned

// _swap_back_to_native_stack(sp_loc: Address)
//                            X0
begin_func muentry_swap_back_to_native_stack
          # SP = *sp_loc
          LDR X9, [X0]
          MOV SP, X9

          pop_callee_saved
          # Restore the previouse frame (and return to its return location)
          exit_frame
          RET
end_func muentry_swap_back_to_native_stack

# _get_current_frame_bp() -> Address
#                             X0
begin_func get_current_frame_bp
          MOV X0, FP
          RET
end_func get_current_frame_bp

# muentry_throw_exception(obj: Address)
#                         X0
begin_func muentry_throw_exception
         # save all callee-saved registers and pass tham as argument 2
         push_pair LR, FP
         MOV FP, SP
         push_callee_saved
         MOV X1, FP // X1 is the frame pointer
         BL throw_exception_internal
         # won't return
end_func muentry_throw_exception
# _exception_restore(dest: Address, frame_cursor: *const Word, sp: Address) -> !
#                    X0             X1                         X2
begin_func exception_restore
          SUB X1, X1, #144 // Skip to the bottom of the frame cursor
          // load each callee saved register relative to the stack pointer
          pop_callee_saved X1
          pop_pair FP, LR, X1
          MOV SP, X2
          BR X0
end_func exception_restore

/*
# swap to the new stack whilst passing values and saving the old stack
# muentry_swapstack_ret_pass(new_stack args..., new_sp: Address, old_sp_loc: &mut Adress)
#                          X0 ... X7  [x8]            X9               X10
begin_func muentry_swapstack_ret_pass
    enter_frame
    push_callee_saved

    # Save the current stack pointer
    MOV X11, SP
    STR X11, [X10]

    # Swap to new stack
    MOV SP, X9

# On the new stack, reverse the above
    pop_callee_saved
    exit_frame
    RET
end_func muentry_swapstack_ret_pass

# Same as swapstack_ret_pass except will throw an exception to the new stack instead of passing values
# muentry_swapstack_ret_throw(exception: Address, new_sp: Address, old_sp_loc: &mut Adress)
#                          X0                  X1               X2
begin_func muentry_swapstack_ret_throw
    enter_frame
    push_callee_saved

    # Save the current stack pointer
    MOV X3, SP
    STR X3, [X2]

    # Swap to new stack
    MOV SP, X1

    # The new stack will have the same layout as the stack when muentry_throw_exception
    # calls throw_exception_internal, so we can do that directly here

    # Add the total size pushed by 'push_callee_saved' to get the FP for the new stack
    ADD FP, SP, 144
    MOV X1, FP
    BL throw_exception_internal

    # won't return
end_func muentry_swapstack_ret_throw

# swap to the new stack whilst passing values and killing the old stack
# muentry_swapstack_kill_pass(new_stack args..., new_sp: Address, old_stack: *mut MuStack)
#                              X0 ... X7 [x8]            X9              X10
begin_func muentry_swapstack_kill_pass
    # Swap to new stack
    MOV SP, X9

    mov_args_to_callee_saved
    MOV X0, X10
    B muentry_kill_stack
    mov_callee_saved_to_args

    # On the new stack, reverse muentry_swapstack_ret_pass
    pop_callee_saved
    exit_frame
    RET
end_func muentry_swapstack_kill_pass

# Same as muentry_swapstack_kill_pass except will throw an exception to the new stack instead of passing values
# muentry_swapstack_kill_throw(exception: Address, new_sp: Address, old_stack: *mut MuStack)
#                          X0                  X1               X2
begin_func muentry_swapstack_kill_throw
    # Swap to new stack
    MOV SP, X1

    mov_args_to_callee_saved
    MOV X0, X9
    B muentry_kill_stack
    mov_callee_saved_to_args

    # The new stack will have the same layout as the stack when muentry_throw_exception
    # calls throw_exception_internal, so we can do that directly here

    # Add the total size pushed by 'push_callee_saved' to get the FP for the new stack
    ADD FP, SP, 144
    MOV X1, FP
    BL throw_exception_internal

    # won't return
end_func muentry_swapstack_kill_throw
*/