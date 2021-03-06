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

#define CNAME(n) n

  .macro begin_func n
  .text
  .globl CNAME(\n)
  .type CNAME(\n),@function
  .balign 16
CNAME(\n):
  .endm

  .macro end_func n
  .size CNAME(\n), .-CNAME(\n)
  .endm

; vim: ft=asm
