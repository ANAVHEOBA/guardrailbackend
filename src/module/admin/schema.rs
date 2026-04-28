use serde::Serialize;

use crate::module::{
    admin::model::{AdminProfile, AdminUploadAssetRecord},
    auth::schema::{
        AuthResponse, UserResponse, WalletChallengeRequest, WalletChallengeResponse,
        WalletConnectRequest,
    },
};

pub type AdminWalletChallengeRequest = WalletChallengeRequest;
pub type AdminWalletChallengeResponse = WalletChallengeResponse;
pub type AdminWalletConnectRequest = WalletConnectRequest;
pub type AdminAuthResponse = AuthResponse;

#[derive(Debug, Serialize)]
pub struct AdminMeResponse {
    pub user: UserResponse,
    pub monad_chain_id: i64,
}

impl AdminMeResponse {
    pub fn from_profile(profile: AdminProfile, monad_chain_id: i64) -> Self {
        Self {
            user: UserResponse::from_parts(profile.user, Some(profile.wallet)),
            monad_chain_id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminImageUploadResponse {
    pub asset: AdminImageAssetResponse,
}

#[derive(Debug, Serialize)]
pub struct AdminImageAssetResponse {
    pub id: String,
    pub storage_provider: String,
    pub bucket_name: String,
    pub scope: String,
    pub file_name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub cid: String,
    pub ipfs_url: String,
    pub gateway_url: String,
    pub created_at: String,
}

impl AdminImageUploadResponse {
    pub fn from_record(record: AdminUploadAssetRecord) -> Self {
        Self {
            asset: AdminImageAssetResponse {
                id: record.id.to_string(),
                storage_provider: record.storage_provider,
                bucket_name: record.bucket_name,
                scope: record.scope,
                file_name: record.file_name,
                content_type: record.content_type,
                size_bytes: record.size_bytes,
                cid: record.cid,
                ipfs_url: record.ipfs_url,
                gateway_url: record.gateway_url,
                created_at: record.created_at.to_rfc3339(),
            },
        }
    }
}
