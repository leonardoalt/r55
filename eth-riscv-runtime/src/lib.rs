#![no_std]
#![no_main]
#![feature(alloc_error_handler, maybe_uninit_write_slice, round_char_boundary)]

use core::arch::asm;
use core::panic::PanicInfo;
use core::slice;
pub use riscv_rt::entry;
use alloy_core::primitives::{Address, B256};

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

pub fn keccak256(offset: u64, size: u64) -> B256 {
    let first: u64;
    let second: u64;
    let third: u64;
    let fourth: u64;

    unsafe {
        asm!(
            "ecall",
            in("a0") offset,
            in("a1") size,
            lateout("a0") first,
            lateout("a1") second,
            lateout("a2") third,
            lateout("a3") fourth,
            in("t0") u32::from(Syscall::Keccak256),
            options(nostack, preserves_flags)
        );
    }

    let mut bytes = [0u8; 32];

    bytes[0..8].copy_from_slice(&first.to_be_bytes());
    bytes[8..16].copy_from_slice(&second.to_be_bytes());
    bytes[16..24].copy_from_slice(&third.to_be_bytes());
    bytes[24..32].copy_from_slice(&fourth.to_be_bytes());

    B256::from_slice(&bytes)
}

pub fn msg_sender() -> Address {
    let first: u64;
    let second: u64;
    let third: u64;
    unsafe {
        asm!("ecall", lateout("a0") first, lateout("a1") second, lateout("a2") third, in("t0") u32::from(Syscall::Caller));
    }
    let mut bytes = [0u8; 20];
    bytes[0..8].copy_from_slice(&first.to_be_bytes());
    bytes[8..16].copy_from_slice(&second.to_be_bytes());
    bytes[16..20].copy_from_slice(&third.to_be_bytes()[..4]);
    Address::from_slice(&bytes)
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
