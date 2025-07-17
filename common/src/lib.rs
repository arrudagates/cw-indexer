use bigdecimal::BigDecimal;
use chrono::{offset::Utc, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub hash: String,
    pub parent_hash: String,
    pub number: i64,
    pub timestamp: DateTime<Utc>,
    pub miner: String,
    pub gas_used: BigDecimal,
    pub gas_limit: BigDecimal,
    pub tx_count: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub block_hash: String,
    pub block_number: i64,
    pub from_address: String,
    pub to_address: Option<String>,
    pub value: BigDecimal,
    pub gas_price: Option<BigDecimal>,
    pub gas_used: Option<BigDecimal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Log {
    pub id: i32,
    pub tx_hash: String,
    pub address: String,
    pub topic0: Option<String>,
    pub topic1: Option<String>,
    pub topic2: Option<String>,
    pub topic3: Option<String>,
    pub data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub id: i32,
    pub tx_hash: String,
    pub token_address: String,
    pub from_address: String,
    pub to_address: String,
    pub value: Option<BigDecimal>,
    pub token_id: Option<BigDecimal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenBalance {
    pub id: i32,
    pub owner_address: String,
    pub token_address: String,
    pub amount: BigDecimal,
    pub token_id: Option<BigDecimal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionDetail {
    pub transaction: Transaction,
    pub logs: Vec<Log>,
    pub token_transfers: Vec<TokenTransfer>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountDetail {
    pub address: String,
    pub token_balances: Vec<TokenBalance>,
}
