_start:
	// constructor()
	// load small binary into address &P
	// prepend 0xffff (RISCV identifier)
	li t0 0
	li a0 &P
	li a1 &P.len
	ecall // returns runtime contract
