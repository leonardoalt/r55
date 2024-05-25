#![no_std]
#![no_main]

use eth_riscv_runtime::return_riscv;

#[no_mangle]
pub extern "C" fn _start() {
    //decode constructor arguments
    //constructor(ars);
    let runtime: &[u8] = include_bytes!("../target/riscv64imac-unknown-none-elf/release/runtime");

    // Assuming we store the result in memory and return its address and offset
    let result_ptr = runtime.as_ptr() as u64;
    let result_len = runtime.len() as u64;

    return_riscv(result_ptr, result_len);
}
