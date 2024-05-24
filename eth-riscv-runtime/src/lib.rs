#![no_std]
#![feature(
    start,
    alloc_error_handler,
    maybe_uninit_write_slice,
    round_char_boundary
)]

use core::arch::asm;
use core::panic::PanicInfo;
use core::slice;

mod alloc;
pub mod types;

pub trait Contract {
    fn call(&self);
}

pub unsafe fn slice_from_raw_parts(address: usize, length: usize) -> &'static [u8] {
    slice::from_raw_parts(address as *const u8, length)
}

#[panic_handler]
unsafe fn panic(panic: &PanicInfo<'_>) -> ! {
    static mut IS_PANICKING: bool = false;

    if !IS_PANICKING {
        IS_PANICKING = true;

        //print!("{panic}\n");
    } else {
        //print_str("Panic handler has panicked! Things are very dire indeed...\n");
    }

    asm!("unimp");
    loop {}
}

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

pub fn revert() {
    unsafe {
        asm!("ecall", in("t0") u32::from(Syscall::Revert));
    }
}
