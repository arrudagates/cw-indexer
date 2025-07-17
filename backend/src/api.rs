use crate::{
    db::DbPool,
    models,
    schema::{self, blocks, logs, token_balances, token_transfers, transactions},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use common::{AccountDetail, Block, TransactionDetail};
use diesel::prelude::*;

/// Handler to get the 20 most recent blocks.
pub async fn get_recent_blocks(State(pool): State<DbPool>) -> Result<Json<Vec<Block>>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch data using Diesel's model
    let results = blocks::table
        .order(blocks::number.desc())
        .limit(20)
        .load::<models::Block>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert from Diesel model to common API model
    let blocks_api: Vec<common::Block> = results.into_iter().map(Into::into).collect();

    Ok(Json(blocks_api))
}

/// Handler to get the details of a single block by its hash.
pub async fn get_block_details(
    State(pool): State<DbPool>,
    Path(hash): Path<String>,
) -> Result<Json<Block>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let block_result = blocks::table
        .filter(blocks::hash.eq(hash))
        .first::<models::Block>(&mut conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match block_result {
        Some(block) => Ok(Json(block.into())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Handler to get the details of a single transaction by its hash.
pub async fn get_transaction_details(
    State(pool): State<DbPool>,
    Path(hash): Path<String>,
) -> Result<Json<TransactionDetail>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Find the transaction
    let tx_result = transactions::table
        .filter(transactions::hash.eq(&hash))
        .first::<models::Transaction>(&mut conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if tx_result.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let tx = tx_result.unwrap();

    // Find associated logs
    let log_results = logs::table
        .filter(logs::tx_hash.eq(&hash))
        .load::<models::Log>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Find associated token transfers
    let transfer_results = token_transfers::table
        .filter(token_transfers::tx_hash.eq(&hash))
        .load::<models::TokenTransfer>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = TransactionDetail {
        transaction: tx.into(),
        logs: log_results.into_iter().map(Into::into).collect(),
        token_transfers: transfer_results.into_iter().map(Into::into).collect(),
    };

    Ok(Json(response))
}

/// Handler to get the details of an account, including token balances.
pub async fn get_account_details(
    State(pool): State<DbPool>,
    Path(address): Path<String>,
) -> Result<Json<AccountDetail>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check if account exists
    let account_exists: bool = diesel::select(diesel::dsl::exists(
        schema::accounts::table.filter(schema::accounts::address.eq(&address)),
    ))
    .get_result(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !account_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    // Find token balances
    let balances = token_balances::table
        .filter(token_balances::owner_address.eq(&address))
        .load::<models::TokenBalance>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = AccountDetail {
        address,
        token_balances: balances.into_iter().map(Into::into).collect(),
    };

    Ok(Json(response))
}

pub async fn get_transactions_for_block(
    State(pool): State<DbPool>,
    Path(hash): Path<String>,
) -> Result<Json<Vec<common::Transaction>>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = schema::transactions::table
        .filter(schema::transactions::block_hash.eq(hash))
        .load::<models::Transaction>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let transactions_api: Vec<common::Transaction> = results.into_iter().map(Into::into).collect();

    Ok(Json(transactions_api))
}

impl From<models::Block> for common::Block {
    fn from(b: models::Block) -> Self {
        Self {
            hash: b.hash,
            parent_hash: b.parent_hash,
            number: b.number,
            timestamp: b.timestamp,
            miner: b.miner,
            gas_used: b.gas_used,
            gas_limit: b.gas_limit,
            tx_count: b.tx_count,
        }
    }
}

impl From<models::Transaction> for common::Transaction {
    fn from(t: models::Transaction) -> Self {
        Self {
            hash: t.hash,
            block_hash: t.block_hash,
            block_number: t.block_number,
            from_address: t.from_address,
            to_address: t.to_address,
            value: t.value,
            gas_price: t.gas_price,
            gas_used: t.gas_used,
        }
    }
}

impl From<models::Log> for common::Log {
    fn from(l: models::Log) -> Self {
        Self {
            id: l.id,
            tx_hash: l.tx_hash,
            address: l.address,
            topic0: l.topic0,
            topic1: l.topic1,
            topic2: l.topic2,
            topic3: l.topic3,
            data: l.data,
        }
    }
}

impl From<models::TokenTransfer> for common::TokenTransfer {
    fn from(t: models::TokenTransfer) -> Self {
        Self {
            id: t.id,
            tx_hash: t.tx_hash,
            token_address: t.token_address,
            from_address: t.from_address,
            to_address: t.to_address,
            value: t.value,
            token_id: t.token_id,
        }
    }
}

impl From<models::TokenBalance> for common::TokenBalance {
    fn from(b: models::TokenBalance) -> Self {
        Self {
            id: b.id,
            owner_address: b.owner_address,
            token_address: b.token_address,
            amount: b.amount,
            token_id: b.token_id,
        }
    }
}
