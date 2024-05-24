#![no_std]
#![no_main]

use eth_riscv_runtime::Contract;
use erc20::ERC20;

#[no_mangle]
pub extern "C" fn _start()
{
    let contract = ERC20::default();
    contract.call();
}
