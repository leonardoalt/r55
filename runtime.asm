_start:
	mstore 0, 5 // mem[0] = 5
	mov t0 0
	mov a0 0
	mov a1 8
	ecall // return(0, 8), returns 0x0000000000000005 to the host

