-- Your SQL goes here
CREATE TABLE blocks (
    hash TEXT PRIMARY KEY,
    parent_hash TEXT NOT NULL,
    number BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    miner TEXT NOT NULL,
    gas_used NUMERIC NOT NULL,
    gas_limit NUMERIC NOT NULL,
    base_fee_per_gas NUMERIC,
    extra_data TEXT NOT NULL,
    tx_count INTEGER NOT NULL
);
