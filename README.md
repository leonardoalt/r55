# R55

R55 is an experimental Ethereum Execution Environment that seamlessly
integrates RISCV smart contracts alongside traditional EVM smart contracts.
This dual support operates over the same Ethereum state, and communication
happens via ABI-encoded calls.

On the high level, R55 enables the use of pure Rust smart contracts, opening
the door for a vast Rust developer community to engage in Ethereum development
with minimal barriers to entry, and increasing language and compiler diversity.

On the low level, RISCV code allows for optimization opportunities distinct
from the EVM, including the use of off-the-shelf ASICs. This potential for
performance gains can be particularly advantageous in specialized domains.

# Off-the-shelf tooling

R55 relies on standard tooling that programmers are used to, such as Rust,
Cargo and LLVM. This directly enables tooling such as linters, static
analyzers, testing, fuzzing, and formal verification tools to be applied to
these smart contracts without extra development and research.

# Pure & Clean Rust Smart Contracts

Differently from other platforms that offer Rust smart contracts, R55 lets the
user code in [no\_std] Rust without weird edges. The code below implements a
basic ERC20 token with infinite minting for testing.
Because the `struct` and `impl` are just Rust code, the user can write normal
tests and run them natively (as long as they don't need Ethereum host
functions).  Note that [alloy-rs](https://github.com/alloy-rs/) types work
out-of-the-box.

```rust
#![no_std]
#![no_main]

use core::default::Default;

use contract_derive::contract;
use eth_riscv_runtime::types::Mapping;

use alloy_core::primitives::Address;

#[derive(Default)]
pub struct ERC20 {
    balance: Mapping<Address, u64>,
}

#[contract]
impl ERC20 {
    pub fn balance_of(&self, owner: Address) -> u64 {
        self.balance.read(owner)
    }

    pub fn transfer(&self, from: Address, to: Address, value: u64) {
        let from_balance = self.balance.read(from);
        let to_balance = self.balance.read(to);

        if from == to || from_balance < value {
            revert();
        }

        self.balance.write(from, from_balance - value);
        self.balance.write(to, to_balance + value);
    }

    pub fn mint(&self, to: Address, value: u64) -> u64 {
        let to_balance = self.balance.read(to);
        self.balance.write(to, to_balance + value);
        self.balance.read(to)
    }
}
```

The macro `#[contract]` above is the only special treatment the user needs to
apply to their code. Specifically, it is responsible for the init code
(deployer), and for creating the function dispatcher based on the given
methods.
Note that Rust `pub` methods are exposed as public functions in the deployed
contract, similarly to Solidity's `public` functions.

# Client Integration

R55 is a fork of [revm](https://github.com/bluealloy/revm) without any API
changes.  Therefore it can be used seamlessly in Anvil/Reth to deploy a
testnet/network with support to RISCV smart contracts.
Nothing has to be changed in how transactions are handled or created.

# Relevant Links

- [revm-R55](https://github.com/r0qs/revm)
- [rvemu-R55](https://github.com/lvella/rvemu)
- [R55 Ethereum Runtime](https://github.com/leonardoalt/r55/tree/main/eth-riscv-runtime)
- [R55 Compiler](https://github.com/leonardoalt/r55/tree/main/r55)