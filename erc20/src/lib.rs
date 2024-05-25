#![no_std]
#![no_main]

use core::default::Default;

use contract_derive::contract;
use eth_riscv_runtime::{revert, slice_from_raw_parts, types::Mapping, Contract};

#[derive(Default)]
pub struct ERC20 {
    balance: Mapping<u64, u64>,
}

#[contract]
impl ERC20 {
    pub fn balance_of(&self, owner: u64) -> u64 {
        self.balance.read(owner)
    }

    pub fn transfer(&self, from: u64, to: u64, value: u64) {
        let from_balance = self.balance.read(from);
        let to_balance = self.balance.read(to);

        if from == to || from_balance < value {
            revert();
        }

        self.balance.write(from, from_balance - value);
        self.balance.write(to, to_balance + value);
    }
}
