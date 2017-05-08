#include "asm_common.S.inc"

FP .req X29
LR .req X30

.macro push_pair src1 src2 stack=SP
STP \src2 , \src1, [ \stack ,#-16]
.endm

.macro pop_pair dest1 dest2 stack=SP
LDP \dest1 , \dest2 , [ \stack ],#16
.endm

.macro enter_frame
push_pair LR, FP
MOV FP, SP
.endm

.macro exit_frame
pop_pair FP, LR
.endm

.macro push_callee_saved stack=SP
push_pair D8, D9, \stack
push_pair D10, D11, \stack
push_pair D12, D13, \stack
push_pair D14, D15, \stack

push_pair X19, X20, \stack
push_pair X21, X22, \stack
push_pair X23, X24, \stack
push_pair X25, X26, \stack
push_pair X27, X28, \stack
.endm

.macro pop_callee_saved stack=SP
pop_pair X28, X27, \stack
pop_pair X26, X25, \stack
pop_pair X24, X23, \stack
pop_pair X22, X21, \stack
pop_pair X20, X19, \stack

pop_pair D15, D14, \stack
pop_pair D13, D12, \stack
pop_pair D11, D10, \stack
pop_pair D9, D8, \stack
.endm

.macro pop_arguments stack=SP // 16
pop_pair x7, x6, \stack
pop_pair x5, x4, \stack
pop_pair x3, x2, \stack
pop_pair x1, x0, \stack

pop_pair d7, d6, \stack
pop_pair d5, d4, \stack
pop_pair d3, d2, \stack
pop_pair d1, d0, \stack
.endm

# swap_stack_to(new_sp: Address, entry: Address, old_sp_loc: Address)
#               X0               X1              X2
begin_func swap_to_mu_stack
          # -- on old stack --
          enter_frame

          # push all callee-saved registers
          push_callee_saved

          # save sp to arg2
          MOV X9, SP
          STR X9, [X2]

          # switch to new stack
          MOV SP, X0
          # save entry function in %X9
          MOV X9, X1

          # -- on new stack --
          # arguments (reverse order of thread.rs - runtime_load_args)
          pop_arguments

          # at this point new stack is clean (no intermediate values)
          MOV FP, SP

          # Set an invalid return address, in case the entry function tries to return
          MOV LR, #0

          # start the entry function
          RET X9
end_func swap_to_mu_stack

# _swap_back_to_native_stack(sp_loc: Address)
#                            X0
begin_func muentry_swap_back_to_native_stack
          # SP <- *sp_loc
          LDR X9, [X0]
          MOV SP, X9

          pop_callee_saved
          # Restore the previouse frame (but discard the LR, so we don't return the the wrong place)
          pop_pair FP, XZR
          RET
end_func muentry_swap_back_to_native_stack

# _get_current_frame_rbp() -> Address
#                             X0
begin_func get_current_frame_rbp
          MOV X0, FP
          RET
end_func get_current_frame_rbp

# muentry_throw_exception(obj: Address)
#                         X0
begin_func muentry_throw_exception
         # save all callee-saved

         push_callee_saved

         # %rsp points to somewhere, pass this as 2nd argument
         MOV X1, SP

#if defined (__linux__)
         B CNAME(throw_exception_internal)
#else
         #error "Only Linux is supported. "
#endif
         # won't return

# _exception_restore(dest: Address, callee_saved: *const Word, rsp: Address) -> !
#                    X0             X1                         X2
# callee_saved: [rbx, rbp, r12, r13, r14, r15]
begin_func exception_restore
          // load each callee saved register relative to the stack pointer
          pop_callee_saved X1

          MOV SP, X2
          BR X0
end_func exception_restore

# fake_swap_mu_thread(old_sp_loc: Address)
#                     X0
# (we do not actually swap stack, but we make the stack the same
# as if they are native stack that have been swapped out, so that
# when THREADEXIT (swap_back_to_native_stack) is called, we won't panic

# this function is untested!!!
begin_func fake_swap_mu_thread
    enter_frame

    push_callee_saved

    # save old sp to thread field
    MOV X9, SP
    STR X9, [X0]

    # return to caller, but preserve those pushed values (since THREADEXIT will pick them up)
    RET
end_func fake_swap_mu_thread
