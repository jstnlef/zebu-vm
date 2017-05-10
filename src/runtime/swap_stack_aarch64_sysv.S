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
          //enter_frame // Unncesary Note saves the return address of the function that called this function

          # If entry tries to return (IT SHOULDN'T) it will return to entry_returned
          ADR LR, entry_returned

          # branch to entry
          RET X9
end_func swap_to_mu_stack

# For debuging only, executes
begin_func entry_returned
          HLT #0
end_func entry_returned


# _swap_back_to_native_stack(sp_loc: Address)
#                            X0
begin_func muentry_swap_back_to_native_stack
          # SP = *sp_loc
          LDR X9, [X0]
          MOV SP, X9

          pop_callee_saved
          # Restore the previouse frame (and return to its return location)
          exit_frame
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
         push_pair LR, FP
         push_callee_saved

         # SP points to the callee saved registers, pass this as 2nd argument
         MOV X1, SP

#if defined (__linux__)
         B throw_exception_internal
#else
         #error "Only Linux is supported. "
#endif
         # won't return

# _exception_restore(dest: Address, callee_saved: *const Word, rsp: Address) -> !
#                    X0             X1                         X2
# callee_saved: [x28, x27, x26, x25, x24, x23, x22, x21, x20, x19, d15, d14, d13, d12, d11, d10, d9, d8, fp, lr]
begin_func exception_restore
          // load each callee saved register relative to the stack pointer
          pop_callee_saved X1
          pop_pair FP, LR, X1 // TODO: Should we be restoring the link register?

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
