-- Your SQL goes here
CREATE TABLE token_balances (
    id SERIAL PRIMARY KEY,
    owner_address TEXT NOT NULL,
    token_address TEXT NOT NULL,
    amount NUMERIC NOT NULL,
    token_id NUMERIC, -- Distinguishes NFTs
    UNIQUE (owner_address, token_address, token_id)
);
