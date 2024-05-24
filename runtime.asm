	.text
_start:
	li t0 0
	li a0 ret_var
	li a1 8
	ecall // return(0, 8), returns 0x0000000000000005 to the host

	.data
	.align 3
ret_var:
	.dword 5
