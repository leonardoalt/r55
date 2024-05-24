#![no_std]

//use core::marker::PhantomData;
//use core::default::Default;
use core::arch::asm;

extern crate alloc;
//use alloc::string::String;

use eth_riscv_syscalls::Syscall;

pub fn return_riscv(addr: u64, offset: u64) {
    unsafe {
        asm!("ecall", in("a0") addr, in("a1") offset, in("t0") u32::from(Syscall::Return));
    }
}

pub fn sload(key: u64) -> u64 {
    let value: u64;
    unsafe {
        asm!("ecall", lateout("a0") value, in("a0") key, in("t0") u32::from(Syscall::SLoad));
    }
    value
}

pub fn sstore(key: u64, value: u64) {
    unsafe {
        asm!("ecall", in("a0") key, in("a1") value, in("t0") u32::from(Syscall::SStore));
    }
}

/*
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
*/
