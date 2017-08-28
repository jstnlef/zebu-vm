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

# starts a muthread that passes values to the target
# muthread_start_normal(new_sp: Address, old_sp_loc: Address)
#                      X0             , X1
begin_func muthread_start_normal
    enter_frame
    push_callee_saved

    // Save the current SP to old_sp_loc
    MOV X11, SP
    STR X11, [X1]

    // Swap to the new stack
    MOV SP, X0

    // Pop the argument registers from the new stack
    LDP D1, D0, [SP, #14*8 ]
    LDP D3, D2, [SP, #12*8 ]
    LDP D5, D4, [SP, #10*8 ]
    LDP D7, D6, [SP, #8*8 ]
    LDP X1, X0, [SP, #6*8]
    LDP X3, X2, [SP, #4*8]
    LDP X5, X4, [SP, #2*8]
    LDP X7, X6, [SP, #0*8]
    ADD SP, SP, #16*8

    // Jump to the new stack
    exit_frame
    BR LR
end_func muthread_start_normal

# starts a muthread with an exception thrown
# muthread_start_exceptional(exception: Address, new_sp: Address, old_sp_loc: &mut Adress)
#                            X0                  X1               X2
begin_func muthread_start_exceptional
    enter_frame
    push_callee_saved

    // Save the current SP to old_sp_loc
    MOV X11, SP
    STR X11, [X1]

    // Swap to the new stack
    MOV SP, X1
    SUB SP, SP, #144 // Alocate space for callee saved registers
    B throw_exception_internal
    // We won't be coming back...
end_func muthread_start_exceptional

# restores the thread
#               (new_sp: Address)
#               X0
begin_func muentry_thread_exit
	# Rust code will be responsible for actually killing the stack
    MOV SP, X0

    // Do the inverse of 'muthread_*'
    pop_callee_saved
    exit_frame
    BR LR
end_func muentry_thread_exit