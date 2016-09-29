# swap_stack_to(new_sp: Address, entry: Address, old_sp_loc: Address)
#               %rdi             %rsi            %rdx
.globl _swap_to_mu_stack
_swap_to_mu_stack:
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
          add $48, %rsp
          # at this point new stack is clean (no intermediate values)

          movq %rsp, %rbp

          # push an empty pointer to stack, if entry fucntion tries to return, it causes a segfault
          pushq $0
          # push entry function and start it
          pushq %rax
          ret

# _swap_back_to_native_stack(sp_loc: Address)
#                            %rdi
.globl _muentry_swap_back_to_native_stack
_muentry_swap_back_to_native_stack:
          movq 0(%rdi), %rsp

          popq %r15
          popq %r14
          popq %r13
          popq %r12
          popq %rbx

          popq %rbp
          ret

# _get_current_frame_rbp() -> Address
.globl _get_current_frame_rbp
_get_current_frame_rbp:
          movq %rbp, %rax
          ret
