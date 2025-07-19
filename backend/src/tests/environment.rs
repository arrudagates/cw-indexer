use crate::{
    db::DbPool,
    models::{Account, Block, NewLog, NewTokenTransfer, Transaction},
    schema::{accounts, blocks, logs, token_transfers, transactions},
    MIGRATIONS,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    Connection, RunQueryDsl,
};
use diesel_migrations::MigrationHarness;
use std::{env, str::FromStr};
use url::Url;

pub struct TestDb {
    pub pool: DbPool,
    db_name: String,
    admin_conn_url: String,
    pub db_url: String,
}

impl TestDb {
    /// Creates a new test database and returns a `TestDb` manager instance.
    pub fn new() -> Self {
        dotenvy::dotenv().ok();
        let database_url =
            env::var("POSTGRES_ADMIN_URL").expect("POSTGRES_ADMIN_URL must be set for tests");

        let db_name = format!("test_db_{}", uuid::Uuid::new_v4());

        // Connect to the default `postgres` database to perform admin tasks.
        let mut admin_url = Url::parse(&database_url).expect("Invalid DATABASE_URL format");
        let admin_conn_url = admin_url.to_string();
        admin_url.set_path("/postgres");
        let mut admin_conn = PgConnection::establish(admin_url.as_str())
            .unwrap_or_else(|_| panic!("Failed to connect to 'postgres' database"));

        // Create the new database.
        diesel::sql_query(format!(r#"CREATE DATABASE "{}""#, db_name))
            .execute(&mut admin_conn)
            .unwrap_or_else(|_| panic!("Failed to create test database: {}", db_name));

        // Establish a connection pool to the newly created database.
        let mut test_db_url = Url::parse(&database_url).unwrap();
        test_db_url.set_path(&db_name);
        let manager = ConnectionManager::<PgConnection>::new(test_db_url.to_string());
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create connection pool for test database");

        // Run migrations on the new database.
        let mut migration_conn = pool.get().expect("Failed to get connection for migrations");
        migration_conn
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations on test database");

        println!("✅ Created test database: {}", db_name);

        Self {
            pool,
            db_name,
            admin_conn_url,
            db_url: test_db_url.to_string(),
        }
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let mut admin_url = Url::parse(&self.admin_conn_url).unwrap();
        admin_url.set_path("/postgres");
        let mut admin_conn = PgConnection::establish(admin_url.as_str())
            .unwrap_or_else(|_| panic!("Failed to connect to 'postgres' database for cleanup"));

        // Forcefully terminate any active connections to the test database.
        let disconnect_query = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
            self.db_name
        );
        if let Err(e) = diesel::sql_query(&disconnect_query).execute(&mut admin_conn) {
            eprintln!(
                "Warning: Could not terminate connections to test database {}: {}",
                self.db_name, e
            );
        }

        // Drop the database.
        let drop_query = format!(r#"DROP DATABASE IF EXISTS "{}""#, self.db_name);
        diesel::sql_query(&drop_query)
            .execute(&mut admin_conn)
            .unwrap_or_else(|e| panic!("Failed to drop test database {}: {}", self.db_name, e));

        println!("🗑️ Dropped test database: {}", self.db_name);
    }
}

/// Inserts a mock block with deterministic data into the database.
///
/// Hashes are generated predictably based on the block number.
pub fn create_mock_block(conn: &mut PgConnection, block_number: i64) -> Block {
    let block = Block {
        hash: format!("0x{:064x}", block_number),
        parent_hash: format!("0x{:064x}", block_number - 1),
        number: block_number,
        timestamp: DateTime::parse_from_rfc3339("2025-07-19T12:00:00-03:00")
            .unwrap()
            .with_timezone(&Utc),
        miner: "0x1111111111111111111111111111111111111111".to_string(),
        gas_used: BigDecimal::from(1_500_000),
        gas_limit: BigDecimal::from(30_000_000),
        base_fee_per_gas: Some(BigDecimal::from(50)),
        extra_data: "0x00".to_string(),
        tx_count: 1,
    };
    diesel::insert_into(blocks::table)
        .values(&block)
        .execute(conn)
        .expect("Failed to insert mock block");
    block
}

/// Inserts a mock transaction with deterministic data into the database.
///
/// The hash is generated predictably from the block number and transaction index.
pub fn create_mock_transaction(
    conn: &mut PgConnection,
    block: &Block,
    tx_index: i32,
) -> Transaction {
    let tx = Transaction {
        hash: format!("0x{:063x}{:x}", block.number, tx_index),
        block_hash: block.hash.clone(),
        block_number: block.number,
        from_address: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        to_address: Some("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string()),
        value: BigDecimal::from_str("1000000000000000000").unwrap(), // 1 ETH
        gas_price: Some(BigDecimal::from(20)),
        gas_used: Some(BigDecimal::from(21000)),
        nonce: block.number + i64::from(tx_index),
        position: tx_index,
    };
    diesel::insert_into(transactions::table)
        .values(&tx)
        .execute(conn)
        .expect("Failed to insert mock transaction");
    tx
}

/// Inserts a mock log with deterministic data into the database.
pub fn create_mock_log(conn: &mut PgConnection, tx: &Transaction) {
    let log = NewLog {
        tx_hash: &tx.hash,
        address: "0xcccccccccccccccccccccccccccccccccccccccc",
        topic0: Some("0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"),
        topic1: Some("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
        topic2: None,
        topic3: None,
        data: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    };
    diesel::insert_into(logs::table)
        .values(&log)
        .execute(conn)
        .expect("Failed to insert mock log");
}

/// Inserts a mock token transfer with deterministic data into the database.
pub fn create_mock_token_transfer(conn: &mut PgConnection, tx: &Transaction) {
    let v = BigDecimal::from(500);

    let transfer = NewTokenTransfer {
        tx_hash: &tx.hash,
        token_address: "0xcccccccccccccccccccccccccccccccccccccccc", // Same as log address for consistency
        from_address: &tx.from_address,
        to_address: tx.to_address.as_ref().unwrap(),
        value: Some(&v),
        token_id: None,
    };
    diesel::insert_into(token_transfers::table)
        .values(&transfer)
        .execute(conn)
        .expect("Failed to insert mock token transfer");
}

/// Inserts a mock account into the database with a deterministic timestamp.
pub fn create_mock_account(conn: &mut PgConnection, address: &str) {
    let account = Account {
        address: address.to_string(),
        created_at: DateTime::parse_from_rfc3339("2025-07-19T12:00:00-03:00")
            .unwrap()
            .with_timezone(&Utc),
    };
    diesel::insert_into(accounts::table)
        .values(&account)
        .on_conflict_do_nothing()
        .execute(conn)
        .expect("Failed to insert mock account");
}
