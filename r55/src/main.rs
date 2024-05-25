use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{
        address, keccak256, ruint::Uint, AccountInfo, Address, Bytecode, Bytes, ExecutionResult,
        Output, TransactTo, U256,
    },
    Evm,
};

fn main() {
    const CONTRACT_ADDR: Address = address!("0d4a11d5EEaaC28EC3F61d100daF4d40471f1852");
    let mut db = CacheDB::new(EmptyDB::default());

    let og_bytecode: &[u8] = include_bytes!("../../c-runtime-examples/sstore-and-sload-example");
    let mut new_bytecode = vec![0xff];
    new_bytecode.extend_from_slice(og_bytecode);

    // Fill database:
    let bytecode = Bytes::from(new_bytecode);
    let account = AccountInfo::new(
        Uint::from(10),
        0,
        keccak256(CONTRACT_ADDR),
        Bytecode::new_raw(bytecode),
    );

    db.insert_account_info(CONTRACT_ADDR, account);

    let mut evm = Evm::builder()
        .with_db(db.clone())
        .modify_tx_env(|tx| {
            tx.caller = address!("0000000000000000000000000000000000000001");
            tx.transact_to = TransactTo::Call(CONTRACT_ADDR);
            tx.data = Bytes::new();
            tx.value = U256::from(0);
        })
        .build();

    let ref_tx = evm.transact().unwrap();
    let result = ref_tx.result;

    match result {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => println!("Value: {:?}", value),
        result => panic!("Unexpected result: {:?}", result),
    };

    let account_db = &db.accounts[&CONTRACT_ADDR];
    println!("Account storage: {:?}", account_db.storage);
    let slot_42 = account_db.storage[&U256::from(42)];

    assert_eq!(slot_42.as_limbs()[0], 0xdeadbeaf);
}
