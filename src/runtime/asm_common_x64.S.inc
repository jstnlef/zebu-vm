// This file provides a template so that we can define functions in ASM and also
// let the symbols be recognised in both Linux and OSX

#if defined (__linux__)

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

#elif defined (__APPLE__)

#define CNAME(n) _##n

  .macro begin_func n
  .text
  .globl CNAME(\n)
  .balign 4
CNAME(\n):
  .endm

  .macro end_func n
  .endm

#else
  #error "Only Linux and OSX are supported."
#endif

; vim: ft=asm
