use super::environment::{
    create_mock_account, create_mock_block, create_mock_log, create_mock_token_transfer,
    create_mock_transaction, TestDb,
};
use crate::{
    api::{
        get_account_details, get_block_details, get_recent_blocks, get_transaction_details,
        get_transactions_for_block,
    },
    schema::token_balances,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use std::str::FromStr;

#[tokio::test]
async fn test_get_recent_blocks() {
    let db = TestDb::new();
    let mut conn = db.pool.get().unwrap();

    // Case 1: Empty database should return an empty list.
    let result = get_recent_blocks(State(db.pool.clone())).await;
    assert_eq!(result.unwrap().0.len(), 0);

    // Case 2: Database with a few blocks.
    create_mock_block(&mut conn, 101);
    create_mock_block(&mut conn, 102);
    create_mock_block(&mut conn, 103);

    let result = get_recent_blocks(State(db.pool.clone())).await.unwrap();
    let blocks = result.0;

    assert_eq!(blocks.len(), 3);
    // Blocks should be ordered by number descending.
    assert_eq!(blocks[0].number, 103);
    assert_eq!(blocks[1].number, 102);
    assert_eq!(blocks[2].number, 101);
    assert_eq!(blocks[0].hash, format!("0x{:064x}", 103));
    assert_eq!(
        blocks[0].miner,
        "0x1111111111111111111111111111111111111111"
    );
}

#[tokio::test]
async fn test_get_block_details() {
    let db = TestDb::new();
    let mut conn = db.pool.get().unwrap();

    // Case 1: Block not found.
    let result = get_block_details(
        State(db.pool.clone()),
        Path("0xnonexistenthash".to_string()),
    )
    .await;
    assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);

    // Case 2: Block is found.
    let mock_block = create_mock_block(&mut conn, 100);
    let result = get_block_details(State(db.pool.clone()), Path(mock_block.hash.clone()))
        .await
        .unwrap();
    let block_detail = result.0;

    assert_eq!(block_detail.hash, mock_block.hash);
    assert_eq!(block_detail.number, 100);
    assert_eq!(
        block_detail.miner,
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(block_detail.tx_count, 1);
}

#[tokio::test]
async fn test_get_transactions_for_block() {
    let db = TestDb::new();
    let mut conn = db.pool.get().unwrap();

    // Case 1: Block hash does not exist, should return empty list.
    let result = get_transactions_for_block(
        State(db.pool.clone()),
        Path("0xnonexistenthash".to_string()),
    )
    .await
    .unwrap();
    assert_eq!(result.0.len(), 0);

    // Case 2: Block with multiple transactions.
    let mock_block = create_mock_block(&mut conn, 200);
    let tx1 = create_mock_transaction(&mut conn, &mock_block, 0);
    let tx2 = create_mock_transaction(&mut conn, &mock_block, 1);

    let result = get_transactions_for_block(State(db.pool.clone()), Path(mock_block.hash.clone()))
        .await
        .unwrap();
    let transactions = result.0;

    assert_eq!(transactions.len(), 2);
    assert_eq!(transactions[0].hash, tx1.hash);
    assert_eq!(transactions[1].hash, tx2.hash);
    assert_eq!(
        transactions[0].from_address,
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
}

#[tokio::test]
async fn test_get_transaction_details() {
    let db = TestDb::new();
    let mut conn = db.pool.get().unwrap();

    // Case 1: Transaction not found.
    let result =
        get_transaction_details(State(db.pool.clone()), Path("0xnonexistenttx".to_string())).await;
    assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);

    // Case 2: Transaction with logs and transfers is found.
    let mock_block = create_mock_block(&mut conn, 300);
    let mock_tx = create_mock_transaction(&mut conn, &mock_block, 0);
    create_mock_log(&mut conn, &mock_tx);
    create_mock_token_transfer(&mut conn, &mock_tx);

    let result = get_transaction_details(State(db.pool.clone()), Path(mock_tx.hash.clone()))
        .await
        .unwrap();
    let tx_detail = result.0;

    assert_eq!(tx_detail.transaction.hash, mock_tx.hash);
    assert_eq!(tx_detail.logs.len(), 1);
    assert_eq!(tx_detail.token_transfers.len(), 1);

    // Assert log details
    assert_eq!(
        tx_detail.logs[0].address,
        "0xcccccccccccccccccccccccccccccccccccccccc"
    );
    assert_eq!(
        tx_detail.logs[0].topic0,
        Some("0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_string())
    );

    // Assert token transfer details
    assert_eq!(
        tx_detail.token_transfers[0].token_address,
        "0xcccccccccccccccccccccccccccccccccccccccc"
    );
    assert_eq!(
        tx_detail.token_transfers[0].value,
        Some(BigDecimal::from(500))
    );
}

#[tokio::test]
async fn test_get_account_details() {
    let db = TestDb::new();
    let mut conn = db.pool.get().unwrap();
    let owner_address = "0xowneraddress0000000000000000000000000000";

    // Case 1: Account not found.
    let result = get_account_details(State(db.pool.clone()), Path(owner_address.to_string())).await;
    assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);

    // Case 2: Account exists with various token balances.
    create_mock_account(&mut conn, owner_address);

    // Insert an ERC20-like token balance.
    let token1_addr = "0x1111111111111111111111111111111111111111";
    diesel::insert_into(token_balances::table)
        .values((
            token_balances::owner_address.eq(owner_address),
            token_balances::token_address.eq(token1_addr),
            token_balances::amount.eq(BigDecimal::from_str("123.45").unwrap()),
        ))
        .execute(&mut conn)
        .unwrap();

    // Insert an NFT-like balance (token_id is not null).
    let token2_addr = "0x2222222222222222222222222222222222222222";
    diesel::insert_into(token_balances::table)
        .values((
            token_balances::owner_address.eq(owner_address),
            token_balances::token_address.eq(token2_addr),
            token_balances::amount.eq(BigDecimal::from(1)),
            token_balances::token_id.eq(Some(BigDecimal::from(999))),
        ))
        .execute(&mut conn)
        .unwrap();

    let result = get_account_details(State(db.pool.clone()), Path(owner_address.to_string()))
        .await
        .unwrap();
    let account_detail = result.0;

    assert_eq!(account_detail.address, owner_address);
    assert_eq!(account_detail.token_balances.len(), 2);

    // Verify balances (order isn't guaranteed, so find each one).
    let token1_balance = account_detail
        .token_balances
        .iter()
        .find(|b| b.token_address == token1_addr)
        .unwrap();
    assert_eq!(
        token1_balance.amount,
        BigDecimal::from_str("123.45").unwrap()
    );
    assert!(token1_balance.token_id.is_none());

    let token2_balance = account_detail
        .token_balances
        .iter()
        .find(|b| b.token_address == token2_addr)
        .unwrap();
    assert_eq!(token2_balance.amount, BigDecimal::from(1));
    assert_eq!(token2_balance.token_id, Some(BigDecimal::from(999)));
}
