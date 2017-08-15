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

# Swapstack internals
.macro stack_pass new_sp
    MOV SP, \new_sp
# On the new stack, reverse the above
    pop_callee_saved
    exit_frame
    RET
.endm

.macro stack_throw new_sp
    MOV SP, \new_sp
    # The new stack will have the same layout as the stack when muentry_throw_exception
    # calls throw_exception_internal, so we can do that directly here

    # Add the total size pushed by 'push_callee_saved' to get the FP for the new stack
    ADD FP, SP, 144
    MOV X1, FP
    BL throw_exception_internal
.endm

.macro stack_ret old_sp
    enter_frame
    push_callee_saved

    MOV X11, SP
    STR X11, [\old_sp]
.endm

.macro stack_kill old_sp
    mov_args_to_callee_saved
    MOV X0, \old_sp
    B muentry_kill_stack
    mov_callee_saved_to_args
.endm

# starts a muthread that passes values to the target (with the given memory area containg values of argument registers)
#               (new_sp: Address, old_sp_loc: Address)
#               X0                  X1,
begin_func muthread_start_pass
    stack_ret X1
    MOV X9, X0 # X1 will be overriden by the next instructions
    load_arguments X9
    stack_pass X9
end_func muthread_start_pass


# Same as muentry_swapstack_ret_throw
# muthread_start_throw(exception: Address, new_sp: Address, old_sp_loc: &mut Adress)
#                          X0                  X1               X2
begin_func muthread_start_throw
    stack_ret X2
    stack_throw X1
end_func muthread_start_throw

# restores the thread
#               (new_sp: Address)
#               X0
begin_func muentry_thread_exit
	# Rust code will be responsible for actually killing the stack
    stack_pass X0
end_func muentry_thread_exit

# swap to the new stack whilst passing values and saving the old stack
# muentry_swapstack_ret_pass(new_stack args..., new_sp: Address, old_sp_loc: &mut Adress)
#                          X0 ... X7  [x8]            X9               X10
begin_func muentry_swapstack_ret_pass
    stack_ret X10
    stack_pass X9
end_func muentry_swapstack_ret_pass

# Same as swapstack_ret_pass except will throw an exception to the new stack instead of passing values
# muentry_swapstack_ret_throw(exception: Address, new_sp: Address, old_sp_loc: &mut Adress)
#                          X0                  X1               X2
begin_func muentry_swapstack_ret_throw
    stack_ret X2
    stack_throw X1
end_func muentry_swapstack_ret_throw

# swap to the new stack whilst passing values and killing the old stack
# muentry_swapstack_kill_pass(new_stack args..., new_sp: Address, old_stack: *mut MuStack)
#                              X0 ... X7 [x8]            X9              X10
begin_func muentry_swapstack_kill_pass
    stack_kill X10
    stack_pass X9
end_func muentry_swapstack_kill_pass

# Same as muentry_swapstack_kill_pass except will throw an exception to the new stack instead of passing values
# muentry_swapstack_kill_throw(exception: Address, new_sp: Address, old_stack: *mut MuStack)
#                          X0                  X1               X2
begin_func muentry_swapstack_kill_throw
    stack_kill X2
    stack_throw X1
end_func muentry_swapstack_kill_throw
