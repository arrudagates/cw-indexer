// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (address) {
        address -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    blocks (hash) {
        hash -> Text,
        parent_hash -> Text,
        number -> Int8,
        timestamp -> Timestamptz,
        miner -> Text,
        gas_used -> Numeric,
        gas_limit -> Numeric,
        base_fee_per_gas -> Nullable<Numeric>,
        extra_data -> Text,
        tx_count -> Int4,
    }
}

diesel::table! {
    logs (id) {
        id -> Int4,
        tx_hash -> Text,
        address -> Text,
        topic0 -> Nullable<Text>,
        topic1 -> Nullable<Text>,
        topic2 -> Nullable<Text>,
        topic3 -> Nullable<Text>,
        data -> Text,
    }
}

diesel::table! {
    token_balances (id) {
        id -> Int4,
        owner_address -> Text,
        token_address -> Text,
        amount -> Numeric,
        token_id -> Nullable<Numeric>,
    }
}

diesel::table! {
    token_transfers (id) {
        id -> Int4,
        tx_hash -> Text,
        token_address -> Text,
        from_address -> Text,
        to_address -> Text,
        value -> Nullable<Numeric>,
        token_id -> Nullable<Numeric>,
    }
}

diesel::table! {
    transactions (hash) {
        hash -> Text,
        block_hash -> Text,
        block_number -> Int8,
        from_address -> Text,
        to_address -> Nullable<Text>,
        value -> Numeric,
        gas_price -> Nullable<Numeric>,
        gas_used -> Nullable<Numeric>,
        nonce -> Int8,
        position -> Int4,
    }
}

diesel::joinable!(logs -> transactions (tx_hash));
diesel::joinable!(token_transfers -> transactions (tx_hash));
diesel::joinable!(transactions -> blocks (block_hash));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    blocks,
    logs,
    token_balances,
    token_transfers,
    transactions,
);
