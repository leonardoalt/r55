	.text
	.globl _start
_start:
	li t0, 0
	la a0, ret_var
	li a1, 8
	ecall # return(ret_var, 8), returns 0x0000000000000005 to the host
	j _start # should be unreachable

	.data
	.align 3
ret_var:
	.dword 5
