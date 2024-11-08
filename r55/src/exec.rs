use alloy_core::primitives::Keccak256;
use eth_riscv_interpreter::setup_from_elf;
use revm::{
    handler::register::EvmHandler,
    interpreter::{
        CallInputs, CallScheme, CallValue, Host, InstructionResult, Interpreter, InterpreterAction,
        InterpreterResult, SharedMemory,
    },
    primitives::{address, Address, Bytes, ExecutionResult, Output, TransactTo, U256},
    Database, Evm, Frame, FrameOrResult, InMemoryDB,
};
use rvemu::{emulator::Emulator, exception::Exception};
use std::{cell::RefCell, ops::Range, rc::Rc, sync::Arc};

pub fn deploy_contract(db: &mut InMemoryDB, bytecode: Bytes) -> Address {
    let mut evm = Evm::builder()
        .with_db(db)
        .modify_tx_env(|tx| {
            tx.caller = address!("0000000000000000000000000000000000000001");
            tx.transact_to = TransactTo::Create;
            tx.data = bytecode;
            tx.value = U256::from(0);
        })
        .append_handler_register(handle_register)
        .build();
    evm.cfg_mut().limit_contract_code_size = Some(usize::MAX);

    let result = evm.transact_commit().unwrap();

    match result {
        ExecutionResult::Success {
            output: Output::Create(_value, Some(addr)),
            ..
        } => {
            println!("Deployed at addr: {:?}", addr);
            addr
        }
        result => panic!("Unexpected result: {:?}", result),
    }
}

pub fn run_tx(db: &mut InMemoryDB, addr: &Address, calldata: Vec<u8>) {
    let mut evm = Evm::builder()
        .with_db(db)
        .modify_tx_env(|tx| {
            tx.caller = address!("0000000000000000000000000000000000000007");
            tx.transact_to = TransactTo::Call(*addr);
            tx.data = calldata.into();
            tx.value = U256::from(0);
        })
        .append_handler_register(handle_register)
        .build();

    let result = evm.transact_commit().unwrap();

    match result {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => println!("Tx result: {:?}", value),
        result => panic!("Unexpected result: {:?}", result),
    };
}

#[derive(Debug)]
struct RVEmu {
    emu: Emulator,
    returned_data_destiny: Option<Range<u64>>,
}

fn riscv_context(frame: &Frame) -> Option<RVEmu> {
    let interpreter = frame.interpreter();
    if matches!(interpreter.bytecode.get(0), Some(0xFF)) {
        let emu = setup_from_elf(&interpreter.bytecode[1..], &interpreter.contract.input);
        Some(RVEmu {
            emu,
            returned_data_destiny: None,
        })
    } else {
        None
    }
}

pub fn handle_register<EXT, DB: Database>(handler: &mut EvmHandler<'_, EXT, DB>) {
    let call_stack = Rc::<RefCell<Vec<_>>>::new(RefCell::new(Vec::new()));

    // create a riscv context on call frame.
    let call_stack_inner = call_stack.clone();
    let old_handle = handler.execution.call.clone();
    handler.execution.call = Arc::new(move |ctx, inputs| {
        let result = old_handle(ctx, inputs);
        if let Ok(FrameOrResult::Frame(frame)) = &result {
            call_stack_inner.borrow_mut().push(riscv_context(frame));
        }
        result
    });

    // create a riscv context on create frame.
    let call_stack_inner = call_stack.clone();
    let old_handle = handler.execution.create.clone();
    handler.execution.create = Arc::new(move |ctx, inputs| {
        let result = old_handle(ctx, inputs);
        if let Ok(FrameOrResult::Frame(frame)) = &result {
            call_stack_inner.borrow_mut().push(riscv_context(frame));
        }
        result
    });

    // execute riscv context or old logic.
    let old_handle = handler.execution.execute_frame.clone();
    handler.execution.execute_frame = Arc::new(move |frame, memory, instraction_table, ctx| {
        let result = if let Some(Some(riscv_context)) = call_stack.borrow_mut().first_mut() {
            execute_riscv(riscv_context, frame.interpreter_mut(), memory, ctx)
        } else {
            old_handle(frame, memory, instraction_table, ctx)?
        };

        // if it is return pop the stack.
        if result.is_return() {
            call_stack.borrow_mut().pop();
        }
        Ok(result)
    });
}

fn execute_riscv(
    rvemu: &mut RVEmu,
    interpreter: &mut Interpreter,
    shared_memory: &mut SharedMemory,
    host: &mut dyn Host,
) -> InterpreterAction {
    let emu = &mut rvemu.emu;
    let returned_data_destiny = &mut rvemu.returned_data_destiny;
    if let Some(destiny) = std::mem::take(returned_data_destiny) {
        let data = emu.cpu.bus.get_dram_slice(destiny).unwrap();
        data.copy_from_slice(shared_memory.slice(0, data.len()))
    }

    let return_revert = |interpreter: &mut Interpreter| {
        InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::Revert,
                // return empty bytecode
                output: Bytes::new(),
                gas: interpreter.gas,
            },
        }
    };

    // Run emulator and capture ecalls
    loop {
        let run_result = emu.start();
        match run_result {
            Err(Exception::EnvironmentCallFromMMode) => {
                let t0: u64 = emu.cpu.xregs.read(5);
                match t0 {
                    0 => {
                        // Syscall::Return
                        let ret_offset: u64 = emu.cpu.xregs.read(10);
                        let ret_size: u64 = emu.cpu.xregs.read(11);
                        let data_bytes = if ret_size != 0 {
                            emu.cpu
                                .bus
                                .get_dram_slice(ret_offset..(ret_offset + ret_size))
                                .unwrap()
                        } else {
                            &mut []
                        };
                        return InterpreterAction::Return {
                            result: InterpreterResult {
                                result: InstructionResult::Return,
                                output: data_bytes.to_vec().into(),
                                gas: interpreter.gas, // FIXME: gas is not correct
                            },
                        };
                    }
                    1 => {
                        // Syscall:SLoad
                        let key: u64 = emu.cpu.xregs.read(10);
                        match host.sload(interpreter.contract.target_address, U256::from(key)) {
                            Some((value, _is_cold)) => {
                                emu.cpu.xregs.write(10, value.as_limbs()[0]);
                            }
                            _ => {
                                return return_revert(interpreter);
                            }
                        }
                    }
                    2 => {
                        // Syscall::SStore
                        let key: u64 = emu.cpu.xregs.read(10);
                        let value: u64 = emu.cpu.xregs.read(11);
                        host.sstore(
                            interpreter.contract.target_address,
                            U256::from(key),
                            U256::from(value),
                        );
                    }
                    3 => {
                        // Syscall::Call
                        let a0: u64 = emu.cpu.xregs.read(10);
                        let address =
                            Address::from_slice(emu.cpu.bus.get_dram_slice(a0..(a0 + 20)).unwrap());
                        let value: u64 = emu.cpu.xregs.read(11);
                        let args_offset: u64 = emu.cpu.xregs.read(12);
                        let args_size: u64 = emu.cpu.xregs.read(13);
                        let ret_offset = emu.cpu.xregs.read(14);
                        let ret_size = emu.cpu.xregs.read(15);

                        *returned_data_destiny = Some(ret_offset..(ret_offset + ret_size));

                        let tx = &host.env().tx;
                        return InterpreterAction::Call {
                            inputs: Box::new(CallInputs {
                                input: emu
                                    .cpu
                                    .bus
                                    .get_dram_slice(args_offset..(args_offset + args_size))
                                    .unwrap()
                                    .to_vec()
                                    .into(),
                                gas_limit: tx.gas_limit,
                                target_address: address,
                                bytecode_address: address,
                                caller: interpreter.contract.target_address,
                                value: CallValue::Transfer(U256::from_le_bytes(
                                    value.to_le_bytes(),
                                )),
                                scheme: CallScheme::Call,
                                is_static: false,
                                is_eof: false,
                                return_memory_offset: 0..ret_size as usize,
                            }),
                        };
                    }
                    4 => {
                        // Syscall::Revert
                        return InterpreterAction::Return {
                            result: InterpreterResult {
                                result: InstructionResult::Revert,
                                output: Bytes::from(0u32.to_le_bytes()), //TODO: return revert(0,0)
                                gas: interpreter.gas, // FIXME: gas is not correct
                            },
                        };
                    }
                    5 => {
                        // Syscall::Caller
                        let caller = interpreter.contract.caller;
                        // Break address into 3 u64s and write to registers
                        let caller_bytes = caller.as_slice();
                        let first_u64 = u64::from_be_bytes(caller_bytes[0..8].try_into().unwrap());
                        emu.cpu.xregs.write(10, first_u64);
                        let second_u64 = u64::from_be_bytes(caller_bytes[8..16].try_into().unwrap());
                        emu.cpu.xregs.write(11, second_u64);
                        let mut padded_bytes = [0u8; 8];
                        padded_bytes[..4].copy_from_slice(&caller_bytes[16..20]);
                        let third_u64 = u64::from_be_bytes(padded_bytes);
                        emu.cpu.xregs.write(12, third_u64);
                    }
                    6 => {
                        // Syscall::Keccak256
                        let offset: u64 = emu.cpu.xregs.read(10);
                        let size: u64 = emu.cpu.xregs.read(11);

                        let data_bytes = if size != 0 {
                            emu.cpu
                                .bus
                                .get_dram_slice(offset..(offset + size))
                                .unwrap()
                        } else {
                            &mut []
                        };

                        let mut hasher = Keccak256::new();
                        hasher.update(data_bytes);
                        let hash: [u8; 32] = hasher.finalize().into();

                        // Write the hash to the emulator's registers
                        emu.cpu.xregs.write(10, u64::from_le_bytes(hash[0..8].try_into().unwrap()));
                        emu.cpu.xregs.write(11, u64::from_le_bytes(hash[8..16].try_into().unwrap()));
                        emu.cpu.xregs.write(12, u64::from_le_bytes(hash[16..24].try_into().unwrap()));
                        emu.cpu.xregs.write(13, u64::from_le_bytes(hash[24..32].try_into().unwrap()));
                    }
                    _ => {
                        println!("Unhandled syscall: {:?}", t0);
                        return return_revert(interpreter);
                    }
                }
            }
            _ => {
                return return_revert(interpreter);
            }
        }
    }
}
