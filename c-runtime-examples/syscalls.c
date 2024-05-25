#include "syscalls.h"

// Wrapper for the `return` syscall
void sys_return(void* data, uint64_t length) {
    register uint64_t a0 asm("a0") = (uint64_t)data;
    register uint64_t a1 asm("a1") = length;
    register uint64_t t0 asm("t0") = SYS_RETURN;
    asm volatile (
        "ecall"
        :
        : "r" (a0), "r" (a1), "r" (t0)
        : "memory"
    );
}

// Wrapper for the `sload` syscall
uint64_t sys_sload(uint64_t key) {
    register uint64_t a0 asm("a0") = key;
    register uint64_t t0 asm("t0") = SYS_SLOAD;
    asm volatile (
        "ecall"
        : "+r" (a0)
        : "r" (t0)
        : "memory"
    );
    return a0;
}

// Wrapper for the `sstore` syscall
void sys_sstore(uint64_t key, uint64_t value) {
    register uint64_t a0 asm("a0") = key;
    register uint64_t a1 asm("a1") = value;
    register uint64_t t0 asm("t0") = SYS_SSTORE;
    asm volatile (
        "ecall"
        :
        : "r" (a0), "r" (a1), "r" (t0)
        : "memory"
    );
}

// Wrapper for the `call` syscall (TODO: Complete args)
void sys_call() {
    register uint64_t t0 asm("t0") = SYS_CALL;
    asm volatile (
        "ecall"
        : 
        : "r" (t0)
        : "memory"
    );
}

// Wrapper for the `revert` syscall
void sys_revert() {
    register uint64_t t0 asm("t0") = SYS_REVERT;
    asm volatile (
        "ecall"
        :
        : "r" (t0)
        : "memory"
    );
}
