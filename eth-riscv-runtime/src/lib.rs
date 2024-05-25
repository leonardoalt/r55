#![no_std]
#![no_main]
#![feature(alloc_error_handler, maybe_uninit_write_slice, round_char_boundary)]

use core::arch::asm;
use core::panic::PanicInfo;
use core::slice;
pub use riscv_rt::entry;

mod alloc;
pub mod types;

pub trait Contract {
    fn call(&self);
    fn call_with_data(&self, calldata: &[u8]);
}

pub unsafe fn slice_from_raw_parts(address: usize, length: usize) -> &'static [u8] {
    slice::from_raw_parts(address as *const u8, length)
}

#[panic_handler]
unsafe fn panic(_panic: &PanicInfo<'_>) -> ! {
    static mut IS_PANICKING: bool = false;

    if !IS_PANICKING {
        IS_PANICKING = true;

        revert();
        // TODO with string
        //print!("{panic}\n");
    } else {
        revert();
        // TODO with string
        //print_str("Panic handler has panicked! Things are very dire indeed...\n");
    }
}

use eth_riscv_syscalls::Syscall;

pub fn return_riscv(addr: u64, offset: u64) -> ! {
    unsafe {
        asm!("ecall", in("a0") addr, in("a1") offset, in("t0") u32::from(Syscall::Return));
    }
    unreachable!()
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

pub fn call(addr: u64, value: u64, in_mem: u64, in_size: u64, out_mem: u64, out_size: u64) {
    unsafe {
        asm!("ecall", in("a0") addr, in("a1") value, in("a2") in_mem, in("a3") in_size, in("a4") out_mem, in("a5") out_size, in("t0") u32::from(Syscall::Call));
    }
}

pub fn revert() -> ! {
    unsafe {
        asm!("ecall", in("t0") u32::from(Syscall::Revert));
    }
    unreachable!()
}

#[allow(non_snake_case)]
#[no_mangle]
fn DefaultHandler() {
    revert();
}

#[allow(non_snake_case)]
#[no_mangle]
fn ExceptionHandler(_trap_frame: &riscv_rt::TrapFrame) -> ! {
    revert();
}
