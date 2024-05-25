#ifndef SYSCALL_WRAPPERS_H
#define SYSCALL_WRAPPERS_H

#include "inttypes.h"

// Enum for syscall opcodes
typedef enum {
    SYS_RETURN = 0,
    SYS_SLOAD  = 1,
    SYS_SSTORE = 2,
    SYS_CALL   = 3,
    SYS_REVERT = 4,
} Syscall;

/**
 * @brief Wrapper for the `return` syscall
 * 
 * @param data Pointer to the data to be returned
 * @param length Length of the data in bytes
 */
void sys_return(void* data, uint64_t length);

/**
 * @brief Wrapper for the `sload` syscall
 * 
 * @param key Storage key to load the value from
 * @return uint64_t Value stored at the given key
 */
uint64_t sys_sload(uint64_t key);

/**
 * @brief Wrapper for the `sstore` syscall
 * 
 * @param key Storage key to store the value at
 * @param value Value to be stored
 */
void sys_sstore(uint64_t key, uint64_t value);

/**
 * @brief Wrapper for the `call` syscall
 * 
 * Placeholder function. Implement as per specific requirements.
 */
void sys_call();

/**
 * @brief Wrapper for the `revert` syscall
 */
void sys_revert();

#endif // SYSCALL_WRAPPERS_H
