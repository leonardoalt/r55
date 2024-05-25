#include "syscalls.h"

void _start() {
    sys_sstore(42, 0xdeadbeef);

    uint64_t value = sys_sload(42);
    if (value != 0xdeadbeef) {
        sys_revert();
    }
}
