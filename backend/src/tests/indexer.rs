use super::environment::{create_mock_block, create_mock_transaction, TestDb};
use crate::{
    indexer::{
        ensure_account_exists, get_latest_indexed_block, process_log, process_token_transfer,
        u256_to_bigdecimal, TRANSFER_EVENT_SIGNATURE,
    },
    models,
    schema::{accounts, logs, token_balances, token_transfers},
};
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use ethers::types::{Address, Bytes, Log as EthersLog, H256, U256};
use std::str::FromStr;

#[test]
fn test_u256_to_bigdecimal_conversion() {
    assert_eq!(
        u256_to_bigdecimal(U256::zero()).unwrap(),
        BigDecimal::from(0)
    );
    assert_eq!(
        u256_to_bigdecimal(U256::from(12345)).unwrap(),
        BigDecimal::from(12345)
    );
    let large_val = U256::from_dec_str("1000000000000000000").unwrap(); // 1 ETH
    assert_eq!(
        u256_to_bigdecimal(large_val).unwrap(),
        BigDecimal::from_str("1000000000000000000").unwrap()
    );
}

#[test]
fn test_ensure_account_exists() {
    let db = TestDb::new();
    let mut conn = db.pool.get().unwrap();
    let addr = "0x1234567890123456789012345678901234567890";

    // First time, account should be created.
    ensure_account_exists(&mut conn, addr).unwrap();
    let count: i64 = accounts::table
        .filter(accounts::address.eq(addr))
        .count()
        .get_result(&mut conn)
        .unwrap();
    assert_eq!(count, 1);

    // Second time, should do nothing.
    ensure_account_exists(&mut conn, addr).unwrap();
    let count: i64 = accounts::table.count().get_result(&mut conn).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_get_latest_indexed_block() {
    let db = TestDb::new();
    let mut conn = db.pool.get().unwrap();

    // On an empty DB, should return None.
    let latest = get_latest_indexed_block(&mut conn).unwrap();
    assert_eq!(latest, None);

    // After inserting a block.
    create_mock_block(&mut conn, 500);
    let latest = get_latest_indexed_block(&mut conn).unwrap();
    assert_eq!(latest, Some(500));

    // After inserting another, higher block.
    create_mock_block(&mut conn, 505);
    let latest = get_latest_indexed_block(&mut conn).unwrap();
    assert_eq!(latest, Some(505));
}

#[test]
fn test_process_log_insertion() {
    let db = TestDb::new();
    let mut conn = db.pool.get().unwrap();
    let mock_block = create_mock_block(&mut conn, 1);
    let mock_tx = create_mock_transaction(&mut conn, &mock_block, 0);

    let eth_log = EthersLog {
        address: "0xcccccccccccccccccccccccccccccccccccccccc"
            .parse()
            .unwrap(),

        topics: vec![
            "0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                .parse()
                .unwrap(),
        ],

        data: "0xff".parse().unwrap(),
        ..Default::default()
    };

    process_log(&mut conn, &mock_tx.hash, &eth_log).unwrap();

    let inserted_log = logs::table.first::<models::Log>(&mut conn).unwrap();
    assert_eq!(inserted_log.tx_hash, mock_tx.hash);
    assert_eq!(
        inserted_log.address,
        "0xcccccccccccccccccccccccccccccccccccccccc"
    );
    assert_eq!(
        inserted_log.topic0,
        Some("0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_string())
    );
    assert_eq!(inserted_log.data, "0xff");
}

#[test]
fn test_process_erc20_token_transfer() {
    let db = TestDb::new();
    let mut conn = db.pool.get().unwrap();
    let mock_block = create_mock_block(&mut conn, 1);
    let mock_tx = create_mock_transaction(&mut conn, &mock_block, 0);

    let from_addr: Address = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        .parse()
        .unwrap();
    let to_addr: Address = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        .parse()
        .unwrap();
    let token_addr: Address = "0xcccccccccccccccccccccccccccccccccccccccc"
        .parse()
        .unwrap();
    let transfer_amount = U256::from(500);

    // Setup initial balance for the sender
    diesel::insert_into(token_balances::table)
        .values((
            token_balances::owner_address.eq(format!("{:#x}", from_addr)),
            token_balances::token_address.eq(format!("{:#x}", token_addr)),
            token_balances::amount.eq(BigDecimal::from(1000)),
        ))
        .execute(&mut conn)
        .unwrap();

    let mut data_bytes = [0u8; 32];
    transfer_amount.to_big_endian(&mut data_bytes);

    let eth_log = EthersLog {
        address: token_addr,
        topics: vec![
            TRANSFER_EVENT_SIGNATURE,
            H256::from(from_addr),
            H256::from(to_addr),
        ],
        data: Bytes::from(data_bytes.to_vec()),
        ..Default::default()
    };

    process_token_transfer(&mut conn, &mock_tx.hash, &eth_log).unwrap();

    // Verify transfer record
    let transfer = token_transfers::table
        .first::<models::TokenTransfer>(&mut conn)
        .unwrap();
    assert_eq!(transfer.value, Some(BigDecimal::from(500)));
    assert!(transfer.token_id.is_none());

    // Verify final balances
    let from_balance: BigDecimal = token_balances::table
        .filter(token_balances::owner_address.eq(format!("{:#x}", from_addr)))
        .select(token_balances::amount)
        .first(&mut conn)
        .unwrap();
    assert_eq!(from_balance, BigDecimal::from(500)); // 1000 - 500

    let to_balance: BigDecimal = token_balances::table
        .filter(token_balances::owner_address.eq(format!("{:#x}", to_addr)))
        .select(token_balances::amount)
        .first(&mut conn)
        .unwrap();
    assert_eq!(to_balance, BigDecimal::from(500));
}

#[test]
fn test_process_erc721_token_transfer() {
    let db = TestDb::new();
    let mut conn = db.pool.get().unwrap();
    let mock_block = create_mock_block(&mut conn, 1);
    let mock_tx = create_mock_transaction(&mut conn, &mock_block, 0);

    let from_addr: Address = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        .parse()
        .unwrap();
    let to_addr: Address = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        .parse()
        .unwrap();
    let token_addr: Address = "0xcccccccccccccccccccccccccccccccccccccccc"
        .parse()
        .unwrap();
    let token_id = U256::from(721);

    // Setup initial balance for the sender
    diesel::insert_into(token_balances::table)
        .values((
            token_balances::owner_address.eq(format!("{:#x}", from_addr)),
            token_balances::token_address.eq(format!("{:#x}", token_addr)),
            token_balances::amount.eq(BigDecimal::from(1)),
            token_balances::token_id.eq(Some(BigDecimal::from(721))),
        ))
        .execute(&mut conn)
        .unwrap();

    let mut token_id_bytes = [0u8; 32];
    token_id.to_big_endian(&mut token_id_bytes);

    let eth_log = EthersLog {
        address: token_addr,
        topics: vec![
            TRANSFER_EVENT_SIGNATURE,
            H256::from(from_addr),
            H256::from(to_addr),
            H256::from(token_id_bytes),
        ],
        ..Default::default()
    };

    process_token_transfer(&mut conn, &mock_tx.hash, &eth_log).unwrap();

    // Verify transfer record
    let transfer = token_transfers::table
        .first::<models::TokenTransfer>(&mut conn)
        .unwrap();
    assert_eq!(transfer.token_id, Some(BigDecimal::from(721)));
    assert_eq!(transfer.value, Some(BigDecimal::from(1)));

    // Verify final balances
    let from_balance: BigDecimal = token_balances::table
        .filter(token_balances::owner_address.eq(format!("{:#x}", from_addr)))
        .filter(token_balances::token_id.eq(Some(BigDecimal::from(721))))
        .select(token_balances::amount)
        .first(&mut conn)
        .unwrap();
    assert_eq!(from_balance, BigDecimal::from(0)); // 1 - 1

    let to_balance: models::TokenBalance = token_balances::table
        .filter(token_balances::owner_address.eq(format!("{:#x}", to_addr)))
        .first(&mut conn)
        .unwrap();
    assert_eq!(to_balance.amount, BigDecimal::from(1));
    assert_eq!(to_balance.token_id, Some(BigDecimal::from(721)));
}
