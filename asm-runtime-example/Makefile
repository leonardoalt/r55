runtime: runtime.s Makefile
	riscv64-unknown-elf-as -o runtime.o runtime.s
	riscv64-unknown-elf-ld -T ../r5-bare-bones.x -static -o runtime runtime.o
	riscv64-unknown-elf-strip -s runtime
