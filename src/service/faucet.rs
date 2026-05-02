use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use ethers_contract::Contract;
use ethers_core::{
    abi::AbiParser,
    types::{Address, U256},
};
use ethers_middleware::SignerMiddleware;
use ethers_providers::{Http, Provider};
use ethers_signers::LocalWallet;
use uuid::Uuid;

use crate::{
    app::AppState,
    config::environment::Environment,
    module::{
        auth::{crud as auth_crud, error::AuthError},
        faucet::{
            crud,
            schema::{FaucetUsdcBalanceResponse, FaucetUsdcResponse},
        },
    },
    service::{
        auth::normalize_wallet_address,
        chain::{admin_signer, parse_contract_address, u256_to_string, wait_for_receipt},
        rpc,
    },
};

pub async fn request_usdc_faucet(
    state: &AppState,
    user_id: Uuid,
) -> Result<FaucetUsdcResponse, AuthError> {
    let wallet = auth_crud::get_wallet_for_user(&state.db, user_id)
        .await?
        .ok_or_else(|| AuthError::forbidden("user wallet is not linked"))?;
    let recipient = normalize_wallet_address(&wallet.wallet_address)?;
    let amount = parse_amount(&state.env.faucet_usdc_amount)
        .map_err(|error| AuthError::internal("invalid FAUCET_USDC_AMOUNT configuration", error))?;

    enforce_cooldown(state, user_id).await?;

    let tx_hash = send_usdc_mint_transaction(&state.env, &recipient, amount).await?;
    let record = crud::insert_faucet_request(
        &state.db,
        user_id,
        &recipient,
        &state.env.payment_token_address,
        &amount.to_string(),
        &tx_hash,
    )
    .await?;
    let balance = read_usdc_balance(state, &recipient).await?;
    let next_available_at =
        record.requested_at + Duration::seconds(state.env.faucet_usdc_cooldown_secs);

    Ok(FaucetUsdcResponse {
        token_address: state.env.payment_token_address.clone(),
        recipient,
        wallet_account_kind: wallet.account_kind,
        amount: record.amount,
        balance: u256_to_string(balance),
        tx_hash: record.tx_hash,
        requested_at: record.requested_at,
        next_available_at,
        cooldown_seconds: state.env.faucet_usdc_cooldown_secs,
    })
}

pub async fn get_mock_usdc_balance(
    state: &AppState,
    address: &str,
) -> Result<FaucetUsdcBalanceResponse, AuthError> {
    let address = normalize_wallet_address(address)?;
    let balance = read_usdc_balance(state, &address).await?;

    Ok(FaucetUsdcBalanceResponse {
        token_address: state.env.payment_token_address.clone(),
        address,
        balance: u256_to_string(balance),
        queried_at: Utc::now(),
    })
}

async fn enforce_cooldown(state: &AppState, user_id: Uuid) -> Result<(), AuthError> {
    let Some(last_request) = crud::get_latest_faucet_request_for_user(&state.db, user_id).await?
    else {
        return Ok(());
    };

    let next_available_at =
        last_request.requested_at + Duration::seconds(state.env.faucet_usdc_cooldown_secs);
    if next_available_at > Utc::now() {
        return Err(AuthError::too_many_requests(format!(
            "faucet is on cooldown until {}",
            next_available_at.to_rfc3339()
        )));
    }

    Ok(())
}

async fn read_usdc_balance(state: &AppState, address: &str) -> Result<U256, AuthError> {
    let contract = read_token_contract(&state.env).await.map_err(|error| {
        AuthError::internal("failed to build faucet token read contract", error)
    })?;
    let address = normalize_wallet_address(address)?;
    let account = address
        .parse::<Address>()
        .map_err(|error| AuthError::internal("invalid faucet balance address", error))?;

    contract
        .method::<_, U256>("balanceOf", account)
        .map_err(|error| AuthError::internal("failed to build balanceOf call", error))?
        .call()
        .await
        .map_err(|error| AuthError::internal("failed to query balanceOf", error))
}

async fn read_token_contract(env: &Environment) -> Result<Contract<Provider<Http>>> {
    let provider = rpc::monad_provider_arc(env).await?;
    Ok(Contract::new(
        parse_contract_address(&env.payment_token_address)?,
        faucet_token_abi()?,
        provider,
    ))
}

async fn write_token_contract(
    env: &Environment,
) -> Result<Contract<SignerMiddleware<Provider<Http>, LocalWallet>>, AuthError> {
    let signer = admin_signer(env).await?;
    Ok(Contract::new(
        parse_contract_address(&env.payment_token_address)
            .map_err(|error| AuthError::internal("invalid payment token address", error))?,
        faucet_token_abi()
            .map_err(|error| AuthError::internal("failed to build faucet token ABI", error))?,
        signer,
    ))
}

async fn send_usdc_mint_transaction(
    env: &Environment,
    recipient: &str,
    amount: U256,
) -> Result<String, AuthError> {
    let contract = write_token_contract(env).await?;
    let recipient = recipient
        .parse::<Address>()
        .map_err(|error| AuthError::internal("invalid faucet recipient address", error))?;
    let call = contract
        .method::<_, ()>("mint", (recipient, amount))
        .map_err(|error| AuthError::internal("failed to build faucet mint transaction", error))?;
    let pending = call
        .send()
        .await
        .map_err(|error| AuthError::internal("failed to submit faucet mint transaction", error))?;

    wait_for_receipt(pending).await
}

fn faucet_token_abi() -> Result<ethers_core::abi::Abi> {
    AbiParser::default()
        .parse(&[
            "function mint(address to, uint256 amount)",
            "function balanceOf(address account) view returns (uint256)",
        ])
        .map_err(Into::into)
}

fn parse_amount(raw: &str) -> Result<U256> {
    let value = raw.trim();
    if value.is_empty() {
        anyhow::bail!("amount is required");
    }

    let amount =
        U256::from_dec_str(value).with_context(|| "amount must be a base-10 integer string")?;
    if amount.is_zero() {
        anyhow::bail!("amount must be greater than zero");
    }

    Ok(amount)
}
