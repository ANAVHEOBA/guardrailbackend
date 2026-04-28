use uuid::Uuid;

use crate::{
    config::db::DbPool,
    module::{
        auth::error::AuthError,
        faucet::model::FaucetRequestRecord,
    },
};

mod sql {
    pub const GET_LATEST_FAUCET_REQUEST_FOR_USER: &str =
        include_str!("sql/get_latest_faucet_request_for_user.sql");
    pub const INSERT_FAUCET_REQUEST: &str = include_str!("sql/insert_faucet_request.sql");
}

pub async fn get_latest_faucet_request_for_user(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Option<FaucetRequestRecord>, AuthError> {
    sqlx::query_as::<_, FaucetRequestRecord>(sql::GET_LATEST_FAUCET_REQUEST_FOR_USER)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AuthError::from)
}

pub async fn insert_faucet_request(
    pool: &DbPool,
    user_id: Uuid,
    wallet_address: &str,
    token_address: &str,
    amount: &str,
    tx_hash: &str,
) -> Result<FaucetRequestRecord, AuthError> {
    sqlx::query_as::<_, FaucetRequestRecord>(sql::INSERT_FAUCET_REQUEST)
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(wallet_address)
        .bind(token_address)
        .bind(amount)
        .bind(tx_hash)
        .fetch_one(pool)
        .await
        .map_err(AuthError::from)
}
