FLAGS=-O3 -ffreestanding -nostdlib -nostartfiles

sstore-and-sload-example: sstore-and-sload-example.c syscalls.c Makefile
	riscv64-unknown-elf-gcc -c _start.s sstore-and-sload-example.c syscalls.c $(FLAGS)
	riscv64-unknown-elf-gcc -o sstore-and-sload-example -T ../r5-bare-bones.x sstore-and-sload-example.o syscalls.o _start.o $(FLAGS) -Wl,-z,max-page-size=16


