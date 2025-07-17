# CW Indexer

This is a simple Ethereum indexer + a web interface.

## Setup

### Database
Create a PostgreSQL database with the name `cw_indexer`.

### .env file
Create a `.env` file in the current directory with the address of a PostgreSQL database and the websocket address of an ethereum RPC:
```
DATABASE_URL=postgres://username:password@localhost/cw_indexer
ETH_RPC_URL=wss://eth.drpc.org
```

## Running the indexer
The indexer has two CLI options
- `--no-indexing`: Serves the API without indexing new blocks
- `--start-block`: On first run, if this option is used the indexer will start from the specified block number, otherwise it will start from block 0

## Building the frontend
To build the frontend use [Trunk](https://trunkrs.dev): `trunk build`.
