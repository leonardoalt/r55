.global _start
.section .text
.align  2
.globl  _start
_start:
    # Set the global pointer (gp)
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop

    # Set the stack pointer (sp)
    la sp, _stack_top

    # Call the main function
    call main

    # Halt the processor (infinite loop)
    wfi
