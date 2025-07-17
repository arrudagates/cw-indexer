-- Your SQL goes here
CREATE TABLE token_transfers (
    id SERIAL PRIMARY KEY,
    tx_hash TEXT NOT NULL REFERENCES transactions(hash) ON DELETE CASCADE,
    token_address TEXT NOT NULL,
    from_address TEXT NOT NULL,
    to_address TEXT NOT NULL,
    value NUMERIC, -- For ERC20
    token_id NUMERIC -- For ERC721
);
