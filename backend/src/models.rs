use crate::schema::{accounts, blocks, logs, token_balances, token_transfers, transactions};
use bigdecimal::BigDecimal;
use chrono::offset::Utc;
use chrono::DateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Block {
    pub hash: String,
    pub parent_hash: String,
    pub number: i64,
    pub timestamp: DateTime<Utc>,
    pub miner: String,
    pub gas_used: BigDecimal,
    pub gas_limit: BigDecimal,
    pub base_fee_per_gas: Option<BigDecimal>,
    pub extra_data: String,
    pub tx_count: i32,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub hash: String,
    pub block_hash: String,
    pub block_number: i64,
    pub from_address: String,
    pub to_address: Option<String>,
    pub value: BigDecimal,
    pub gas_price: Option<BigDecimal>,
    pub gas_used: Option<BigDecimal>,
    pub nonce: i64,
    pub position: i32,
}

#[derive(Queryable, Selectable, Insertable, Clone, Debug, Serialize, Deserialize)]
#[diesel(table_name = logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Log {
    // Note: 'id' is not included in the Insertable model as it's auto-generated
    // by the database. If you need to insert, create a separate NewLog struct.
    // For simplicity here, we assume a simple mapping.
    // To make this struct usable with `insert_into`, the `id` field should be omitted
    // or a separate `NewLog` struct should be created without it.
    pub id: i32,
    pub tx_hash: String,
    pub address: String,
    pub topic0: Option<String>,
    pub topic1: Option<String>,
    pub topic2: Option<String>,
    pub topic3: Option<String>,
    pub data: String,
}

// A version of the Log struct for insertion, without the auto-generated `id`.
#[derive(Insertable)]
#[diesel(table_name = logs)]
pub struct NewLog<'a> {
    pub tx_hash: &'a str,
    pub address: &'a str,
    pub topic0: Option<&'a str>,
    pub topic1: Option<&'a str>,
    pub topic2: Option<&'a str>,
    pub topic3: Option<&'a str>,
    pub data: &'a str,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Account {
    pub address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Insertable, Clone, Debug, Serialize, Deserialize)]
#[diesel(table_name = token_transfers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TokenTransfer {
    pub id: i32,
    pub tx_hash: String,
    pub token_address: String,
    pub from_address: String,
    pub to_address: String,
    pub value: Option<BigDecimal>,    // For ERC20
    pub token_id: Option<BigDecimal>, // For ERC721
}

// A struct for inserting new token transfers.
#[derive(Insertable)]
#[diesel(table_name = token_transfers)]
pub struct NewTokenTransfer<'a> {
    pub tx_hash: &'a str,
    pub token_address: &'a str,
    pub from_address: &'a str,
    pub to_address: &'a str,
    pub value: Option<&'a BigDecimal>,
    pub token_id: Option<&'a BigDecimal>,
}

#[derive(Queryable, Selectable, Insertable, Clone, Debug, Serialize, Deserialize)]
#[diesel(table_name = token_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TokenBalance {
    pub id: i32,
    pub owner_address: String,
    pub token_address: String,
    pub amount: BigDecimal,
    pub token_id: Option<BigDecimal>, // NULL for ERC20, value for ERC721
}
