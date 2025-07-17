-- Your SQL goes here
CREATE TABLE transactions (
    hash TEXT PRIMARY KEY,
    block_hash TEXT NOT NULL REFERENCES blocks(hash) ON DELETE CASCADE,
    block_number BIGINT NOT NULL,
    from_address TEXT NOT NULL,
    to_address TEXT,
    value NUMERIC NOT NULL,
    gas_price NUMERIC,
    gas_used NUMERIC,
    nonce BIGINT NOT NULL,
    position INTEGER NOT NULL
);
