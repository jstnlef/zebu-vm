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

#include "asm_common_x64_sel4_rumprun.S.inc"

# swap_stack_to(new_sp: Address, entry: Address, old_sp_loc: Address)
#               %rdi             %rsi            %rdx
begin_func swap_to_mu_stack
          # -- on old stack --
          # C calling convention
          pushq %rbp
          movq %rsp, %rbp

          # other callee-saved registers
          pushq %rbx
          pushq %r12
          pushq %r13
          pushq %r14
          pushq %r15

          # save sp to %rbx
          movq %rsp, 0(%rdx)

          # switch to new stack
          movq %rdi, %rsp
          # save entry function in %rax
          movq %rsi, %rax

          # -- on new stack --
          # arguments (reverse order of thread.rs - runtime_load_args)
          popq %r9
          popq %r8
          popq %rcx
          popq %rdx
          popq %rsi
          popq %rdi
          movsd 0(%rsp), %xmm7
          movsd 8(%rsp), %xmm6
          movsd 16(%rsp), %xmm5
          movsd 24(%rsp), %xmm4
          movsd 32(%rsp), %xmm3
          movsd 40(%rsp), %xmm2
          movsd 48(%rsp), %xmm1
          movsd 56(%rsp), %xmm0
          add $64, %rsp
          # at this point new stack is clean (no intermediate values)

          movq %rsp, %rbp

          # push an empty pointer to stack, if entry fucntion tries to return, it causes a segfault
          pushq $0
          # push entry function and start it
          pushq %rax
          ret
end_func swap_to_mu_stack

# _swap_back_to_native_stack(sp_loc: Address)
#                            %rdi
begin_func muentry_swap_back_to_native_stack
          movq 0(%rdi), %rsp

          popq %r15
          popq %r14
          popq %r13
          popq %r12
          popq %rbx

          popq %rbp
          ret
end_func muentry_swap_back_to_native_stack

# _get_current_frame_rbp() -> Address
begin_func get_current_frame_rbp
          movq %rbp, %rax
          ret
end_func get_current_frame_rbp

# muentry_throw_exception(obj: Address)
#                         %rdi
begin_func muentry_throw_exception
         # save all callee-saved
         pushq %r15
         pushq %r14
         pushq %r13
         pushq %r12
         pushq %rbp
         pushq %rbx

         # %rsp points to %rbx, pass this as 2nd argument
         movq  %rsp, %rsi

         jmp CNAME(throw_exception_internal@PLT)
         # won't return

# _exception_restore(dest: Address, callee_saved: *const Word, rsp: Address) -> !
#                    %rdi           %rsi                       %rdx
# callee_saved: [rbx, rbp, r12, r13, r14, r15]
begin_func exception_restore
          movq 0(%rsi), %rbx
          movq 8(%rsi), %rbp
          movq 16(%rsi),%r12
          movq 24(%rsi),%r13
          movq 32(%rsi),%r14
          movq 40(%rsi),%r15

          movq %rdx, %rsp
          jmpq *%rdi
end_func exception_restore

# fake_swap_mu_thread(old_sp_loc: Address)
#                     %rdi
# (we do not actually swap stack, but we make the stack the same
# as if they are native stack that have been swapped out, so that
# when THREADEXIT (swap_back_to_native_stack) is called, we won't panic

# this function is untested!!!
begin_func fake_swap_mu_thread
    # save return address
    movq (%rsp), %rax

    pushq %rbp
    movq %rsp, %rbp

    pushq %rbx
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    # save old sp to thread field
    movq %rsp, 0(%rdi)

    # return to caller, but preserve those pushed values (since THREADEXIT will pick them up)
    pushq %rax
    ret