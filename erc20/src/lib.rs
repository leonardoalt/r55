#![no_std]
#![no_main]

use core::default::Default;

use contract_derive::contract;
use eth_riscv_runtime::{types::Mapping, log};

use alloy_core::primitives::{Address, address};

#[derive(Default)]
pub struct ERC20 {
    balance: Mapping<Address, u64>,
}

#[contract]
impl ERC20 {
    pub fn balance_of(&self, owner: Address) -> u64 {
        self.balance.read(owner)
    }

    pub fn transfer(&self, from: Address, to: Address, value: u64) -> bool {
        let from_balance = self.balance.read(from);
        let to_balance = self.balance.read(to);

        if from == to || from_balance < value {
            revert();
        }

        self.balance.write(from, from_balance - value);
        self.balance.write(to, to_balance + value);

        let mut log_data = [0u8; 48];  // 20 + 20 + 8 bytes
        log_data[..20].copy_from_slice(from.as_slice());
        log_data[20..40].copy_from_slice(to.as_slice());
        log_data[40..48].copy_from_slice(&value.to_ne_bytes());
    
        log(
            log_data.as_ptr() as u64,
            log_data.len() as u64,
            0,
            0
        );
        true
    }

    pub fn mint(&self, to: Address, value: u64) -> bool  {
        let owner = msg_sender();
        if owner != address!("0000000000000000000000000000000000000007") {
            revert();
        }

        let to_balance = self.balance.read(to);
        self.balance.write(to, to_balance + value);
        true
    }
}
