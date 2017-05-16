// This file provides a template so that we can define functions in ASM and also
// let the symbols be recognised in both Linux and OSX

#if defined (__linux__)
  .macro begin_func n
  .text
  .globl \n
  .type \n,@function
  .balign 4
\n:
  .endm

  .macro end_func n
  .size \n, .-\n
  .endm

#else
  #error "Only Linux is supported."
#endif

FP .req X29
LR .req X30

.macro push_pair src1 src2 stack=SP
STP \src2 , \src1, [ \stack ,#-16]!
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

.macro pop_arguments stack=SP
pop_pair x7, x6, \stack
pop_pair x5, x4, \stack
pop_pair x3, x2, \stack
pop_pair x1, x0, \stack

pop_pair d7, d6, \stack
pop_pair d5, d4, \stack
pop_pair d3, d2, \stack
pop_pair d1, d0, \stack
.endm
