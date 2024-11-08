mod exec;
use exec::{deploy_contract, run_tx};

use std::fs::File;
use std::io::Read;
use std::process::Command;

use alloy_sol_types::SolValue;
use revm::{
    primitives::{address, keccak256, ruint::Uint, AccountInfo, Address, Bytecode, Bytes, Log},
    InMemoryDB,
};
use alloy_core::hex;

fn compile_runtime(path: &str) -> Result<Vec<u8>, ()> {
    println!("Compiling runtime: {}", path);
    let status = Command::new("cargo")
        .arg("+nightly-2024-02-01")
        .arg("build")
        .arg("-r")
        .arg("--lib")
        .arg("-Z")
        .arg("build-std=core,alloc")
        .arg("--target")
        .arg("riscv64imac-unknown-none-elf")
        .arg("--bin")
        .arg("runtime")
        .current_dir(path)
        .status()
        .expect("Failed to execute cargo command");

    if !status.success() {
        eprintln!("Cargo command failed with status: {}", status);
        std::process::exit(1);
    } else {
        println!("Cargo command completed successfully");
    }

    let path = format!(
        "{}/target/riscv64imac-unknown-none-elf/release/runtime",
        path
    );
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return Err(());
        }
    };

    // Read the file contents into a vector.
    let mut bytecode = Vec::new();
    if let Err(e) = file.read_to_end(&mut bytecode) {
        eprintln!("Failed to read file: {}", e);
        return Err(());
    }

    Ok(bytecode)
}

fn compile_deploy(path: &str) -> Result<Vec<u8>, ()> {
    compile_runtime(path)?;
    println!("Compiling deploy: {}", path);
    let status = Command::new("cargo")
        .arg("+nightly-2024-02-01")
        .arg("build")
        .arg("-r")
        .arg("--lib")
        .arg("-Z")
        .arg("build-std=core,alloc")
        .arg("--target")
        .arg("riscv64imac-unknown-none-elf")
        .arg("--bin")
        .arg("deploy")
        .current_dir(path)
        .status()
        .expect("Failed to execute cargo command");

    if !status.success() {
        eprintln!("Cargo command failed with status: {}", status);
        std::process::exit(1);
    } else {
        println!("Cargo command completed successfully");
    }

    let path = format!(
        "{}/target/riscv64imac-unknown-none-elf/release/deploy",
        path
    );
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return Err(());
        }
    };

    // Read the file contents into a vector.
    let mut bytecode = Vec::new();
    if let Err(e) = file.read_to_end(&mut bytecode) {
        eprintln!("Failed to read file: {}", e);
        return Err(());
    }

    Ok(bytecode)
}

fn add_contract_to_db(db: &mut InMemoryDB, addr: Address, bytecode: Bytes) {
    let account = AccountInfo::new(
        Uint::from(0),
        0,
        keccak256(&bytecode),
        Bytecode::new_raw(bytecode),
    );
    db.insert_account_info(addr, account);
}

fn test_runtime_from_binary() {
    let rv_bytecode = compile_runtime("erc20").unwrap();

    const CONTRACT_ADDR: Address = address!("0d4a11d5EEaaC28EC3F61d100daF4d40471f1852");
    let mut db = InMemoryDB::default();

    let mut bytecode = vec![0xff];
    bytecode.extend_from_slice(&rv_bytecode);

    let bytecode = Bytes::from(bytecode);

    add_contract_to_db(&mut db, CONTRACT_ADDR, bytecode);

    let selector_balance: u32 = 0;
    let selector_mint: u32 = 2;
    let to: Address = address!("0000000000000000000000000000000000000001");
    let value_mint: u64 = 42;
    let mut calldata_balance = to.abi_encode();
    let mut calldata_mint = (to, value_mint).abi_encode();

    let selector_bytes_balance = selector_balance.to_le_bytes().to_vec();
    let mut complete_calldata_balance = selector_bytes_balance;
    complete_calldata_balance.append(&mut calldata_balance);

    let selector_bytes_mint = selector_mint.to_le_bytes().to_vec();
    let mut complete_calldata_mint = selector_bytes_mint;
    complete_calldata_mint.append(&mut calldata_mint);

    run_tx(&mut db, &CONTRACT_ADDR, complete_calldata_mint.clone());
    run_tx(&mut db, &CONTRACT_ADDR, complete_calldata_balance.clone());

    /*
    let account_db = &evm.db().accounts[&CONTRACT_ADDR];
    println!("Account storage: {:?}", account_db.storage);
    let slot_42 = account_db.storage[&U256::from(42)];
    assert_eq!(slot_42.as_limbs()[0], 0xdeadbeef);
    */
}

fn test_runtime(addr: &Address, db: &mut InMemoryDB) {
    let selector_balance: u32 = 0;
    let selector_mint: u32 = 2;
    let to: Address = address!("0000000000000000000000000000000000000001");
    let value_mint: u64 = 42;
    let mut calldata_balance = to.abi_encode();
    let mut calldata_mint = (to, value_mint).abi_encode();

    let selector_bytes_balance = selector_balance.to_le_bytes().to_vec();
    let mut complete_calldata_balance = selector_bytes_balance;
    complete_calldata_balance.append(&mut calldata_balance);

    let selector_bytes_mint = selector_mint.to_le_bytes().to_vec();
    let mut complete_calldata_mint = selector_bytes_mint;
    complete_calldata_mint.append(&mut calldata_mint);

    run_tx(db, addr, complete_calldata_mint.clone());
    run_tx(db, addr, complete_calldata_balance.clone());
}

fn test_deploy() {
    let rv_bytecode = compile_deploy("erc20").unwrap();
    let mut db = InMemoryDB::default();

    let mut bytecode = vec![0xff];
    bytecode.extend_from_slice(&rv_bytecode);

    let bytecode = Bytes::from(bytecode);

    let addr = deploy_contract(&mut db, bytecode);

    test_runtime(&addr, &mut db);
}

//////////////////////////
///     TESTS           //
//////////////////////////

fn parse_hex_result(result: &[u8]) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&result[24..32]); 
    u64::from_be_bytes(bytes)
}

fn parse_bool_result(hex_result: &[u8]) -> bool {
    if hex_result.len() == 32 {
        hex_result[31] == 1
    } else {
        false
    }
}


fn test_erc20_mint_and_transfer() {
    let rv_bytecode = compile_runtime("erc20").unwrap();
    println!("Bytecode length: {}", rv_bytecode.len());

    const CONTRACT_ADDR: Address = address!("0d4a11d5EEaaC28EC3F61d100daF4d40471f1852");
    let mut db = InMemoryDB::default();

    let mut bytecode = vec![0xff];
    bytecode.extend_from_slice(&rv_bytecode);
    let bytecode = Bytes::from(bytecode);
    add_contract_to_db(&mut db, CONTRACT_ADDR, bytecode);

    let selector_balance: u32 = 0;
    let selector_transfer: u32 = 1;
    let selector_mint: u32 = 2;
    
    let address1: Address = address!("0000000000000000000000000000000000000001");
    let address2: Address = address!("0000000000000000000000000000000000000002");
    
    // Mint 100 tokens
    let mint_value: u64 = 100;
    let mut calldata_mint = (address1, mint_value).abi_encode();
    let mut complete_calldata_mint = selector_mint.to_le_bytes().to_vec();
    complete_calldata_mint.append(&mut calldata_mint);
    
    println!("\n=== Minting tokens ===");
    println!("Minting {} tokens to address1...", mint_value);
    let mint_result = run_tx(&mut db, &CONTRACT_ADDR, complete_calldata_mint);
    println!("Mint tx result: {}", parse_bool_result(&mint_result.output));

    // Check initial balance
    let mut calldata_balance = address1.abi_encode();
    let mut complete_calldata_balance = selector_balance.to_le_bytes().to_vec();
    complete_calldata_balance.append(&mut calldata_balance);
    
    println!("\n=== Checking initial balance ===");
    let balance_result = run_tx(&mut db, &CONTRACT_ADDR, complete_calldata_balance.clone());
    let initial_balance = parse_hex_result(&balance_result.output);
    println!("Address1 balance: {} (hex: 0x{})", initial_balance, hex::encode(&balance_result.output));
    assert_eq!(initial_balance, mint_value, "Initial balance should be {}", mint_value);

    // Transfer 30 tokens
    let transfer_value: u64 = 30;
    let mut calldata_transfer = (address1, address2, transfer_value).abi_encode();
    let mut complete_calldata_transfer = selector_transfer.to_le_bytes().to_vec();
    complete_calldata_transfer.append(&mut calldata_transfer);
    
    println!("\n=== Transferring tokens ===");
    println!("Transferring {} tokens from address1 to address2...", transfer_value);
    let transfer_result = run_tx(&mut db, &CONTRACT_ADDR, complete_calldata_transfer);
    println!("Transfer tx result: {}", parse_bool_result(&transfer_result.output));

    // Check final balances
    println!("\n=== Checking final balances ===");
    let balance_result1 = run_tx(&mut db, &CONTRACT_ADDR, complete_calldata_balance.clone());
    let final_balance1 = parse_hex_result(&balance_result1.output);
    println!("Address1 final balance: {} (hex: 0x{})", final_balance1, hex::encode(&balance_result1.output));
    assert_eq!(final_balance1, mint_value - transfer_value, 
        "Address1 should have {} tokens", mint_value - transfer_value);

    let mut calldata_balance2 = address2.abi_encode();
    let mut complete_calldata_balance2 = selector_balance.to_le_bytes().to_vec();
    complete_calldata_balance2.append(&mut calldata_balance2);
    
    let balance_result2 = run_tx(&mut db, &CONTRACT_ADDR, complete_calldata_balance2);
    let final_balance2 = parse_hex_result(&balance_result2.output);
    println!("Address2 final balance: {} (hex: 0x{})", final_balance2, hex::encode(&balance_result2.output));
    assert_eq!(final_balance2, transfer_value, 
        "Address2 should have {} tokens", transfer_value);

    println!("\n=== Test Summary ===");
    println!("✓ Initial mint: {} tokens to address1", mint_value);
    println!("✓ Initial balance of address1: {} tokens", initial_balance);
    println!("✓ Transfer: {} tokens from address1 to address2", transfer_value);
    println!("✓ Final balance of address1: {} tokens", final_balance1);
    println!("✓ Final balance of address2: {} tokens", final_balance2);
}

fn analyze_transfer_log(log: &Log, from: Address, to: Address, value: u64) {
    println!("\n=== Transfer Log Analysis ===");
    
    let data = log.data.data.as_ref();
    
    let from_in_log = &data[..20];
    let to_in_log = &data[20..40];
    let value_in_log = u64::from_ne_bytes(data[40..48].try_into().unwrap());
    
    println!("\n--- Addresses ---");
    println!("From address:");
    println!("  Original: 0x{}", hex::encode(from.as_slice()));
    println!("  In log:   0x{}", hex::encode(from_in_log));
    
    println!("\nTo address:");
    println!("  Original: 0x{}", hex::encode(to.as_slice()));
    println!("  In log:   0x{}", hex::encode(to_in_log));
    
    println!("\n--- Values ---");
    println!("Transferred value:");
    println!("  Expected: {} tokens", value);
    println!("  In log:   {} tokens", value_in_log);
    println!("  Value bytes: 0x{}", hex::encode(&data[40..48]));
    
    // Assertions
    assert_eq!(
        from_in_log,
        from.as_slice(),
        "From address mismatch"
    );
    assert_eq!(
        to_in_log,
        to.as_slice(),
        "To address mismatch"
    );
    assert_eq!(value_in_log, value, "Value mismatch");
}


fn test_transfer_with_logs() {
    let rv_bytecode = compile_runtime("erc20").unwrap();
    const CONTRACT_ADDR: Address = address!("0d4a11d5EEaaC28EC3F61d100daF4d40471f1852");
    let mut db = InMemoryDB::default();

    let mut bytecode = vec![0xff];
    bytecode.extend_from_slice(&rv_bytecode);
    let bytecode = Bytes::from(bytecode);
    add_contract_to_db(&mut db, CONTRACT_ADDR, bytecode);

    let selector_mint: u32 = 2;
    let selector_transfer: u32 = 1;
    
    let address1 = address!("0000000000000000000000000000000000000001");
    let address2 = address!("0000000000000000000000000000000000000002");
    
    // Mint tokens
    let mint_value: u64 = 5000;
    let mut calldata_mint = (address1, mint_value).abi_encode();
    let mut complete_calldata_mint = selector_mint.to_le_bytes().to_vec();
    complete_calldata_mint.append(&mut calldata_mint);
    run_tx(&mut db, &CONTRACT_ADDR, complete_calldata_mint);

    // Transfer tokens
    let transfer_value: u64 = 750;
    let mut calldata_transfer = (address1, address2, transfer_value).abi_encode();
    let mut complete_calldata_transfer = selector_transfer.to_le_bytes().to_vec();
    complete_calldata_transfer.append(&mut calldata_transfer);
    
    let transfer_result = run_tx(&mut db, &CONTRACT_ADDR, complete_calldata_transfer);

    assert_eq!(transfer_result.logs.len(), 1, "Should have exactly 1 log");
    
    analyze_transfer_log(
        &transfer_result.logs[0],
        address1,
        address2,
        transfer_value
    );
}


fn test_transfer_logs_simple() {
    let rv_bytecode = compile_runtime("erc20").unwrap();
    const CONTRACT_ADDR: Address = address!("0d4a11d5EEaaC28EC3F61d100daF4d40471f1852");
    let mut db = InMemoryDB::default();

    let mut bytecode = vec![0xff];
    bytecode.extend_from_slice(&rv_bytecode);
    let bytecode = Bytes::from(bytecode);
    add_contract_to_db(&mut db, CONTRACT_ADDR, bytecode);

    let address1 = address!("0000000000000000000000000000000000000001");
    let address2 = address!("0000000000000000000000000000000000000002");
    
    // Mint
    let selector_mint: u32 = 2;
    let mut calldata_mint = (address1, 5000u64).abi_encode();
    let mut complete_calldata_mint = selector_mint.to_le_bytes().to_vec();
    complete_calldata_mint.append(&mut calldata_mint);
    run_tx(&mut db, &CONTRACT_ADDR, complete_calldata_mint);

    // Transfer
    let selector_transfer: u32 = 1;
    let mut calldata_transfer = (address1, address2, 750u64).abi_encode();
    let mut complete_calldata_transfer = selector_transfer.to_le_bytes().to_vec();
    complete_calldata_transfer.append(&mut calldata_transfer);
    
    let transfer_result = run_tx(&mut db, &CONTRACT_ADDR, complete_calldata_transfer);

    println!("\n=== Transfer Log Output ===");
    for log in transfer_result.logs.iter() {
        let data = log.data.data.as_ref();
        
        let from_hex = hex::encode(&data[..20]);
        let to_hex = hex::encode(&data[20..40]);
        let value = u64::from_ne_bytes(data[40..48].try_into().unwrap());

        println!("From: 0x{}", from_hex);
        println!("To: 0x{}", to_hex);
        println!("Value: {}", value);
    }
}
fn main() {
    test_runtime_from_binary();
    test_deploy();
    test_erc20_mint_and_transfer();
    test_transfer_with_logs();
    test_transfer_logs_simple();

}
