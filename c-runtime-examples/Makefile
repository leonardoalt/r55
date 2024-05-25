FLAGS=-O3 -ffreestanding -nostdlib -nostartfiles

sstore-and-sload-example: sstore-and-sload-example.c syscalls.c
	riscv64-unknown-elf-gcc -c sstore-and-sload-example.c syscalls.c $(FLAGS)
	riscv64-unknown-elf-gcc -o sstore-and-sload-example -T ../r5-bare-bones.x sstore-and-sload-example.o syscalls.o $(FLAGS)


