#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crate::api::{
    get_account_details, get_block_details, get_recent_blocks, get_transaction_details,
    get_transactions_for_block,
};
use axum::{routing::get, Router};
use clap::Parser;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

mod api;
mod db;
mod indexer;
mod models;
mod schema;

#[derive(Parser, Debug)]
#[command()]
struct Cli {
    #[arg(long, default_value_t = false)]
    no_indexing: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Load .env file
    dotenvy::dotenv().ok();

    // Establish DB connection pool
    let pool = db::establish_connection_pool();

    if cli.no_indexing {
        println!("🚫 Indexer is disabled by --no-indexing flag. Running in API-only mode.");
    } else {
        println!("🚀 Starting indexer background task...");
        let indexer_pool = pool.clone();
        tokio::spawn(async move {
            if let Err(e) = indexer::run_indexer(indexer_pool).await {
                eprintln!("Indexer process failed: {}", e);
            }
        });
    }

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/blocks", get(get_recent_blocks))
        .route("/api/block/:hash", get(get_block_details))
        .route(
            "/api/block/:hash/transactions",
            get(get_transactions_for_block),
        )
        .route("/api/tx/:hash", get(get_transaction_details))
        .route("/api/account/:address", get(get_account_details))
        .with_state(pool)
        .layer(cors);

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Backend listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
