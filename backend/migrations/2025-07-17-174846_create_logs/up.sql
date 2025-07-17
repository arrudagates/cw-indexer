-- Your SQL goes here
CREATE TABLE logs (
    id SERIAL PRIMARY KEY,
    tx_hash TEXT NOT NULL REFERENCES transactions(hash) ON DELETE CASCADE,
    address TEXT NOT NULL,
    topic0 TEXT,
    topic1 TEXT,
    topic2 TEXT,
    topic3 TEXT,
    data TEXT NOT NULL
);
