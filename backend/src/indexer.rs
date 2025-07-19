use crate::{
    db::DbPool,
    models::{Account, Block, NewLog, NewTokenTransfer, Transaction},
    schema::{accounts, blocks, logs, token_balances, token_transfers, transactions},
};
use anyhow::Result;
use bigdecimal::BigDecimal;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
};
use ethers::{
    prelude::*,
    types::{Block as EthersBlock, Log as EthersLog, Transaction as EthersTransaction},
};
use std::{str::FromStr, sync::Arc};
use tokio::time::{sleep, Duration};

type EthProvider = Provider<Ws>;
type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

// Standard `Transfer(address,address,uint256)` event signature for ERC20 and ERC721
pub const TRANSFER_EVENT_SIGNATURE: H256 = H256([
    0xdd, 0xf2, 0x52, 0xad, 0x1b, 0xe2, 0xc8, 0x9b, 0x69, 0xc2, 0xb0, 0x68, 0xfc, 0x37, 0x8d, 0xaa,
    0x95, 0x2b, 0xa7, 0xf1, 0x63, 0xc4, 0xa1, 0x16, 0x28, 0xf5, 0x5a, 0x4d, 0xf5, 0x23, 0xb3, 0xef,
]);

/// The main entry point for the indexer.
pub async fn run_indexer(pool: DbPool, start_block: Option<u64>) -> Result<()> {
    let rpc_url = std::env::var("ETH_RPC_URL").expect("ETH_RPC_URL must be set");
    let provider = Arc::new(EthProvider::connect(&rpc_url).await?);
    println!("✅ Indexer connected to Ethereum RPC");

    let mut conn = pool.get()?;
    let mut start_block =
        get_latest_indexed_block(&mut conn)?.unwrap_or(start_block.unwrap_or(0) as i64);
    drop(conn); // Release connection before long-running loop

    println!("🚀 Starting indexer from block {}", start_block);

    loop {
        let latest_block_on_chain = provider.get_block_number().await?.as_u64() as i64;

        if start_block > latest_block_on_chain {
            // Wait for new blocks to be mined
            sleep(Duration::from_secs(5)).await;
            continue;
        }

        println!("Indexing block {}", start_block);

        match provider.get_block_with_txs(start_block as u64).await {
            Ok(Some(block)) => {
                let pool_clone = pool.clone();
                let provider_clone = provider.clone();
                tokio::spawn(async move {
                    if let Err(e) = process_block(pool_clone, provider_clone, block).await {
                        eprintln!("Error processing block {}: {}", start_block, e);
                    }
                });
                start_block += 1;
            }
            Ok(None) => {
                eprintln!("Block {} not found, waiting...", start_block);
                sleep(Duration::from_secs(10)).await;
            }
            Err(e) => {
                eprintln!("Error fetching block {}: {}", start_block, e);
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

/// Processes a single block and its transactions, storing them in the database.
pub async fn process_block(
    pool: DbPool,
    provider: Arc<EthProvider>,
    block: EthersBlock<EthersTransaction>,
) -> Result<()> {
    // Collect all transaction hashes from the block.
    let tx_hashes: Vec<H256> = block.transactions.iter().map(|tx| tx.hash).collect();

    // Create a stream of futures to get all receipts concurrently.
    let receipt_futures = tx_hashes
        .iter()
        .map(|hash| provider.get_transaction_receipt(*hash));

    // Await all futures and collect the results.
    let receipts_results = futures::future::join_all(receipt_futures).await;

    // Combine transactions with their corresponding receipts.
    let mut transactions_with_receipts = Vec::new();
    for (tx, receipt_result) in block.transactions.iter().zip(receipts_results) {
        match receipt_result {
            Ok(Some(receipt)) => transactions_with_receipts.push((tx.clone(), receipt)),
            Ok(None) => return Err(anyhow::anyhow!("Receipt not found for tx {}", tx.hash)),
            Err(e) => return Err(e.into()),
        }
    }

    // Run all database operations in a single synchronous transaction.
    let mut conn = pool.get()?;
    process_block_data(&mut conn, &block, &transactions_with_receipts)?;

    println!(
        "✅ Successfully indexed block {}",
        block.number.unwrap().as_u64()
    );
    Ok(())
}

/// Executes all database writes for a block within a single transaction.
pub fn process_block_data(
    connection: &mut DbConnection,
    block: &EthersBlock<EthersTransaction>,
    transactions_with_receipts: &[(EthersTransaction, TransactionReceipt)],
) -> Result<()> {
    connection.transaction::<_, anyhow::Error, _>(|conn| {
        // Insert Block
        let new_block = Block {
            hash: format!("{:#x}", block.hash.unwrap()),
            parent_hash: format!("{:#x}", block.parent_hash),
            number: block.number.unwrap().as_u64() as i64,
            timestamp: chrono::DateTime::from_timestamp(block.timestamp.as_u64() as i64, 0)
                .unwrap(),
            miner: format!("{:#x}", block.author.unwrap()),
            gas_used: u256_to_bigdecimal(block.gas_used)?,
            gas_limit: u256_to_bigdecimal(block.gas_limit)?,
            base_fee_per_gas: block
                .base_fee_per_gas
                .map(|v| u256_to_bigdecimal(v).unwrap()),
            extra_data: block.extra_data.to_string(),
            tx_count: block.transactions.len() as i32,
        };
        diesel::insert_into(blocks::table)
            .values(&new_block)
            .on_conflict_do_nothing()
            .execute(conn)?;

        // Process all collected transactions and receipts
        for (pos, (tx, receipt)) in transactions_with_receipts.iter().enumerate() {
            let from_addr_str = format!("{:#x}", tx.from);
            let to_addr_str = tx.to.map(|a| format!("{:#x}", a));

            ensure_account_exists(conn, &from_addr_str)?;
            if let Some(to) = &to_addr_str {
                ensure_account_exists(conn, to)?;
            }

            let new_tx = Transaction {
                hash: format!("{:#x}", tx.hash),
                block_hash: format!("{:#x}", block.hash.unwrap()),
                block_number: block.number.unwrap().as_u64() as i64,
                from_address: from_addr_str,
                to_address: to_addr_str,
                value: u256_to_bigdecimal(tx.value)?,
                gas_price: receipt
                    .effective_gas_price
                    .map(|v| u256_to_bigdecimal(v).unwrap()),
                gas_used: receipt.gas_used.map(|v| u256_to_bigdecimal(v).unwrap()),
                nonce: tx.nonce.as_u64() as i64,
                position: pos as i32,
            };

            diesel::insert_into(transactions::table)
                .values(&new_tx)
                .on_conflict_do_nothing()
                .execute(conn)?;

            for log in &receipt.logs {
                process_log(conn, &format!("{:#x}", log.transaction_hash.unwrap()), log)?;
            }
        }
        Ok(())
    })
}

/// Processes a single log entry.
pub fn process_log(connection: &mut DbConnection, tx_hash: &str, log: &EthersLog) -> Result<()> {
    let topic0 = log.topics.first().map(|h| format!("{:#x}", h));
    let topic1 = log.topics.get(1).map(|h| format!("{:#x}", h));
    let topic2 = log.topics.get(2).map(|h| format!("{:#x}", h));
    let topic3 = log.topics.get(3).map(|h| format!("{:#x}", h));

    // Insert the raw log
    let new_log = NewLog {
        tx_hash,
        address: &format!("{:#x}", log.address),
        topic0: topic0.as_deref(),
        topic1: topic1.as_deref(),
        topic2: topic2.as_deref(),
        topic3: topic3.as_deref(),
        data: &log.data.to_string(),
    };
    diesel::insert_into(logs::table)
        .values(&new_log)
        .execute(connection)?;

    // Check if it's a token transfer
    if let Some(topic) = topic0 {
        if H256::from_str(&topic)? == TRANSFER_EVENT_SIGNATURE {
            process_token_transfer(connection, tx_hash, log)?;
        }
    }

    Ok(())
}

/// Processes a decoded Transfer event to update token balances.
pub fn process_token_transfer(
    connection: &mut DbConnection,
    tx_hash: &str,
    log: &EthersLog,
) -> Result<()> {
    if log.topics.len() < 3 {
        return Ok(());
    }

    let token_address = format!("{:#x}", log.address);
    let from_address = format!("{:#x}", Address::from(log.topics[1]));
    let to_address = format!("{:#x}", Address::from(log.topics[2]));

    let (value, token_id) = if log.topics.len() == 4 {
        let value = BigDecimal::from(1);
        let token_id = Some(u256_to_bigdecimal(U256::from_big_endian(
            log.topics[3].as_bytes(),
        ))?);
        (Some(value), token_id)
    } else {
        let value = Some(u256_to_bigdecimal(U256::from_big_endian(&log.data))?);
        (value, None)
    };

    let new_transfer = NewTokenTransfer {
        tx_hash,
        token_address: &token_address,
        from_address: &from_address,
        to_address: &to_address,
        value: value.as_ref(),
        token_id: token_id.as_ref(),
    };
    diesel::insert_into(token_transfers::table)
        .values(&new_transfer)
        .execute(connection)?;

    if from_address != format!("{:#x}", Address::zero()) {
        diesel::update(
            token_balances::table
                .filter(token_balances::owner_address.eq(&from_address))
                .filter(token_balances::token_address.eq(&token_address))
                .filter(token_balances::token_id.is_not_distinct_from(&token_id)),
        )
        .set(token_balances::amount.eq(token_balances::amount - value.as_ref().unwrap()))
        .execute(connection)?;
    }

    diesel::insert_into(token_balances::table)
        .values((
            token_balances::owner_address.eq(&to_address),
            token_balances::token_address.eq(&token_address),
            token_balances::amount.eq(value.as_ref().unwrap()),
            token_balances::token_id.eq(&token_id),
        ))
        .on_conflict((
            token_balances::owner_address,
            token_balances::token_address,
            token_balances::token_id,
        ))
        .do_update()
        .set(token_balances::amount.eq(token_balances::amount + value.as_ref().unwrap()))
        .execute(connection)?;

    Ok(())
}

/// Gets the latest block number from the database.
pub fn get_latest_indexed_block(connection: &mut DbConnection) -> Result<Option<i64>> {
    blocks::table
        .select(diesel::dsl::max(blocks::number))
        .get_result::<Option<i64>>(connection)
        .map_err(Into::into)
}

/// Ensures an account exists in the database, inserting it if it doesn't.
pub fn ensure_account_exists(connection: &mut DbConnection, addr: &str) -> Result<()> {
    let new_account = Account {
        address: addr.to_string(),
        created_at: chrono::Utc::now(),
    };
    diesel::insert_into(accounts::table)
        .values(&new_account)
        .on_conflict_do_nothing()
        .execute(connection)?;
    Ok(())
}

/// Converts a U256 value to a `BigDecimal`.
pub fn u256_to_bigdecimal(value: U256) -> Result<BigDecimal> {
    BigDecimal::from_str(&value.to_string()).map_err(Into::into)
}
