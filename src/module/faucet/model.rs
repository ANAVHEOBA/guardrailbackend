use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct FaucetRequestRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub wallet_address: String,
    pub token_address: String,
    pub amount: String,
    pub tx_hash: String,
    pub requested_at: DateTime<Utc>,
}
