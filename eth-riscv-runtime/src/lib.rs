#![no_std]

use core::marker::PhantomData;
use core::default::Default;

extern crate alloc;
use alloc::string::String;

use eth_riscv_syscalls::Return;

fn sload(key: u64) -> u64 {
    let value: u64;
    unsafe {
        asm!(
            "ecall",
            in("a0") 0,       // syscall number for sload
            in("a1") key,
            out("a0") value,
        );
    }
    value
}

fn sstore(key: u64, value: u64) {
    // TODO implement as asm syscall
}

#[derive(Default)]
struct Mapping<K, V> {
    pd: PhantomData<(K, V)>
}

impl<K: Into<u64>, V: Into<u64> + From<u64>> Mapping<K, V> {
    fn read(&self, key: K) -> V {
        sload(key.into()).into()
    } 

    fn write(&self, key: K, value: V) {
        sstore(key.into(), value.into());
    } 
}
