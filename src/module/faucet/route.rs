use axum::{
    Router, middleware as axum_middleware,
    routing::{get, post},
};

use crate::{
    app::AppState,
    middleware::user::require_auth,
    module::faucet::controller::{faucet_usdc, mock_usdc_balance},
};

pub fn public_router() -> Router<AppState> {
    Router::new().route("/faucet/usdc/balance", get(mock_usdc_balance))
}

pub fn me_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/faucet/usdc", post(faucet_usdc))
        .route_layer(axum_middleware::from_fn_with_state(state, require_auth))
}
