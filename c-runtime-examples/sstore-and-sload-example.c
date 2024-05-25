#include "syscalls.h"

void main() {
    sys_sstore(42, 0xdeadbeef);

    uint64_t value = sys_sload(42);
    if (value != 0xdeadbeef) {
        sys_revert();
    }
    sys_return((void*)0, 0);
}
