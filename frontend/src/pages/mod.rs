mod account_details;
mod block_details;
mod homepage;
mod transaction_details;

pub(crate) use self::{
    account_details::AccountDetailsPage, block_details::BlockDetailsPage, homepage::HomePage,
    transaction_details::TransactionDetailsPage,
};
