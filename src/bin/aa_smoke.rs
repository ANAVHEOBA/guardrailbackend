use anyhow::{Context, Result, anyhow};
use ethers_contract::Contract;
use ethers_core::types::{Address, U256};
use guardrailbackend::{
    config::environment::Environment,
    module::auth::model::NewWalletRecord,
    service::{
        aa::{self, SmartAccountCall, SmartAccountSignerContext},
        liquidity::abi::erc20_abi,
        rpc,
    },
};
use reqwest::Client;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let env = Environment::load()?;
    let http_client = Client::new();
    let wallet = aa::provision_local_smart_account(&env, Uuid::new_v4())
        .await
        .context("failed to provision smoke-test smart account")?;
    let signer = signer_context(wallet)?;
    let call = build_approve_call(&env).await?;

    println!("smart_account={}", signer.wallet_address);
    println!("owner_address={}", signer.owner_address);
    println!("payment_token={}", env.payment_token_address);
    println!("spender={}", env.treasury_address);

    let result = aa::submit_calls(&env, &http_client, &signer, &[call])
        .await
        .context("Monad AA smoke test failed")?;

    println!("tx_hash={}", result.tx_hash);
    Ok(())
}

fn signer_context(wallet: NewWalletRecord) -> Result<SmartAccountSignerContext> {
    Ok(SmartAccountSignerContext {
        wallet_address: wallet.wallet_address,
        owner_address: wallet
            .owner_address
            .ok_or_else(|| anyhow!("smart-account wallet is missing owner_address"))?,
        owner_provider: wallet
            .owner_provider
            .ok_or_else(|| anyhow!("smart-account wallet is missing owner_provider"))?,
        owner_ref: wallet
            .owner_ref
            .ok_or_else(|| anyhow!("smart-account wallet is missing owner_ref"))?,
        factory_address: wallet
            .factory_address
            .ok_or_else(|| anyhow!("smart-account wallet is missing factory_address"))?,
        entry_point_address: wallet
            .entry_point_address
            .ok_or_else(|| anyhow!("smart-account wallet is missing entry_point_address"))?,
        owner_encrypted_private_key: wallet
            .owner_encrypted_private_key
            .ok_or_else(|| anyhow!("smart-account wallet is missing encrypted owner key"))?,
        owner_encryption_nonce: wallet
            .owner_encryption_nonce
            .ok_or_else(|| anyhow!("smart-account wallet is missing encryption nonce"))?,
    })
}

async fn build_approve_call(env: &Environment) -> Result<SmartAccountCall> {
    let provider = rpc::monad_provider_arc(env).await?;
    let token_address = parse_address(&env.payment_token_address)?;
    let spender = parse_address(&env.treasury_address)?;
    let contract = Contract::new(token_address, erc20_abi()?, provider);
    let calldata = contract
        .method::<_, bool>("approve", (spender, U256::from(1_u64)))?
        .calldata()
        .ok_or_else(|| anyhow!("missing ERC20 approve calldata"))?;

    Ok(SmartAccountCall {
        target: token_address,
        data: calldata,
    })
}

fn parse_address(value: &str) -> Result<Address> {
    value
        .parse()
        .map_err(|error| anyhow!("invalid address `{value}`: {error}"))
}
