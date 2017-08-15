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

  .macro global_label n
  .globl \n
\n:
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
push_pair X19, X20, \stack
push_pair X21, X22, \stack
push_pair X23, X24, \stack
push_pair X25, X26, \stack
push_pair X27, X28, \stack

push_pair D8, D9, \stack
push_pair D10, D11, \stack
push_pair D12, D13, \stack
push_pair D14, D15, \stack
.endm

.macro pop_callee_saved stack=SP
pop_pair D15, D14, \stack
pop_pair D13, D12, \stack
pop_pair D11, D10, \stack
pop_pair D9, D8, \stack

pop_pair X28, X27, \stack
pop_pair X26, X25, \stack
pop_pair X24, X23, \stack
pop_pair X22, X21, \stack
pop_pair X20, X19, \stack
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

.macro load_arguments stack=SP
LDP D0, D1, [\stack, #-2*8]
LDP D2, D3, [\stack, #-4*8]
LDP D4, D5, [\stack, #-6*8]
LDP D6, D7, [\stack, #-8*8]
LDP X0, X1, [\stack, #-10*8]
LDP X2, X3, [\stack, #-12*8]
LDP X4, X5, [\stack, #-14*8]
LDP X6, X7, [\stack, #-16*8]
.endm

.macro mov_args_to_callee_saved
MOV X19, X0
MOV X20, X1
MOV X21, X2
MOV X22, X3
MOV X23, X4
MOV X24, X5
MOV X25, X6
MOV X26, X7
FMOV D8, D0
FMOV D9, D1
FMOV D10, D2
FMOV D11, D3
FMOV D12, D4
FMOV D13, D5
FMOV D14, D6
FMOV D15, D7
.endm

.macro mov_callee_saved_to_args
MOV X0, X19
MOV X1, X20
MOV X2, X21
MOV X3, X22
MOV X4, X23
MOV X5, X24
MOV X6, X25
MOV X7, X26
FMOV D0, D8
FMOV D1, D9
FMOV D2, D10
FMOV D3, D11
FMOV D4, D12
FMOV D5, D13
FMOV D6, D14
FMOV D7, D15
.endm