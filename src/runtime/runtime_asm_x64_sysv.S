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

#include "asm_common_x64.S.inc"

# muthread_start_normal(new_sp: Address, old_sp_loc: Address)
#                       %rdi             %rsi
begin_func muthread_start_normal
          # -- on old stack --
          # C calling convention - enter frame
          pushq %rbp
          movq %rsp, %rbp

          # save callee saved registers
          pushq %rbx
          pushq %r12
          pushq %r13
          pushq %r14
          pushq %r15

          # save sp to old_sp_loc
          movq %rsp, 0(%rsi)

          # switch to new stack
          movq %rdi, %rsp

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

          # on stack it look like this
          # SP -> 0
          #       entry_func

          # pop 0 as rbp
          popq %rbp
          # pop entry func to r10 (this is a caller saved register, it is save for us to use)
          popq %r10

          call *%r10
end_func muthread_start_normal

# muentry_thread_exit(old_sp: Address)
#                     %rdi
begin_func muentry_thread_exit
          movq %rdi, %rsp

          popq %r15
          popq %r14
          popq %r13
          popq %r12
          popq %rbx

          popq %rbp
          ret
end_func muentry_thread_exit

# _get_current_frame_bp() -> Address
begin_func get_current_frame_bp
          movq %rbp, %rax
          ret
end_func get_current_frame_bp

# muentry_throw_exception(obj: Address)
#                         %rdi
begin_func muentry_throw_exception
         # save all callee-saved
         pushq %rbp
         movq %rsp, %rbp
         pushq %rbx
         pushq %r12
         pushq %r13
         pushq %r14
         pushq %r15

         # %rsp points to %rbx, pass this as 2nd argument
         movq  %rbp, %rsi

#if defined (__linux__)
         jmp CNAME(throw_exception_internal@PLT)
#elif defined (__APPLE__)
         jmp CNAME(throw_exception_internal)
#else
         #error "Only Linux and OSX are supported. "
#endif
         # won't return

# _exception_restore(dest: Address, callee_saved: *const Word, rsp: Address) -> !
#                    %rdi           %rsi                       %rdx
# callee_saved: [rbx, rbp, r12, r13, r14, r15]
begin_func exception_restore
          movq 0(%rsi), %rbp
          movq -8(%rsi),%rbx
          movq -16(%rsi),%r12
          movq -24(%rsi),%r13
          movq -32(%rsi),%r14
          movq -40(%rsi),%r15

          movq %rdx, %rsp
          jmpq *%rdi
end_func exception_restore