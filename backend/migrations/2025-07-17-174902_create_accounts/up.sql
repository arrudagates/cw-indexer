-- Your SQL goes here
CREATE TABLE accounts (
    address TEXT PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
