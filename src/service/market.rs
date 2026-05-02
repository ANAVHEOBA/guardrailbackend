use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use chrono::DateTime;
use ethers_contract::Contract;
use ethers_core::types::Address;
use ethers_providers::{Http, Provider};
use reqwest::{RequestBuilder, StatusCode};
use rust_decimal::{Decimal, RoundingStrategy};
use serde::Deserialize;
use serde_json::Number;

use crate::{
    app::AppState,
    config::environment::Environment,
    module::{
        auth::error::AuthError,
        market::schema::{
            MarketAmountQuote, PaymentTokenQuoteQuery, PaymentTokenQuoteResponse,
            SupportedMarketCurrenciesResponse,
        },
    },
    service::{
        asset::abi::erc20_abi,
        chain::{format_address, parse_contract_address},
        rpc,
    },
};

const DEFAULT_STABLECOIN_SOURCES: &[(&str, &str)] =
    &[("usdc", "usdc-usd-coin"), ("usdt", "usdt-tether")];

#[derive(Debug)]
struct PaymentTokenMetadata {
    address: String,
    symbol: String,
    decimals: u8,
}

#[derive(Debug)]
struct PaymentTokenUsdQuote {
    coin_id: String,
    usd_per_payment_token: Decimal,
    last_updated_at: Option<i64>,
}

#[derive(Debug)]
struct MarketRate {
    market_currency: String,
    market_currency_per_payment_token: Decimal,
    usd_per_payment_token: Decimal,
    last_updated_at: Option<i64>,
    payment_token_coin_id: String,
}

#[derive(Debug)]
enum MarketCurrencyKind {
    Fiat {
        code_lower: String,
        code_upper: String,
    },
    Stablecoin {
        code_lower: String,
        coin_id: &'static str,
    },
}

#[derive(Debug, Deserialize)]
struct ExchangeRateApiLatestResponse {
    result: Option<String>,
    rates: HashMap<String, Number>,
}

#[derive(Debug, Deserialize)]
struct CoinPaprikaTickerResponse {
    quotes: HashMap<String, CoinPaprikaQuote>,
    last_updated: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CoinPaprikaQuote {
    price: Number,
}

pub async fn get_payment_token_quote(
    state: &AppState,
    query: PaymentTokenQuoteQuery,
) -> Result<PaymentTokenQuoteResponse, AuthError> {
    let payment_token = read_payment_token_metadata(&state.env).await?;
    let market_rate = fetch_market_rate(
        state,
        &payment_token,
        normalize_market_currency(&query.market_currency)?,
    )
    .await?;

    Ok(PaymentTokenQuoteResponse {
        market_currency: market_rate.market_currency,
        payment_token_coin_id: market_rate.payment_token_coin_id,
        payment_token_address: payment_token.address,
        payment_token_symbol: payment_token.symbol,
        payment_token_decimals: payment_token.decimals,
        market_currency_per_payment_token: format_decimal(
            market_rate.market_currency_per_payment_token,
        ),
        usd_per_payment_token: format_decimal(market_rate.usd_per_payment_token),
        last_updated_at: market_rate.last_updated_at,
        amount: build_amount_quote(
            query.amount.as_deref(),
            market_rate.market_currency_per_payment_token,
            payment_token.decimals,
            "amount",
        )?,
        subscription_price: build_amount_quote(
            query.subscription_price.as_deref(),
            market_rate.market_currency_per_payment_token,
            payment_token.decimals,
            "subscription_price",
        )?,
        redemption_price: build_amount_quote(
            query.redemption_price.as_deref(),
            market_rate.market_currency_per_payment_token,
            payment_token.decimals,
            "redemption_price",
        )?,
    })
}

pub async fn get_supported_market_currencies(
    state: &AppState,
) -> Result<SupportedMarketCurrenciesResponse, AuthError> {
    let url = format!(
        "{}/latest/USD",
        state.env.exchange_rate_api_base_url.trim_end_matches('/'),
    );

    let response = exchange_rate_api_request(state.http_client.get(url)).await?;
    let payload = response
        .json::<ExchangeRateApiLatestResponse>()
        .await
        .map_err(|error| {
            AuthError::internal("failed to decode ExchangeRate-API latest response", error)
        })?;

    ensure_exchange_rate_api_success(&payload)?;

    let mut supported_currencies = payload
        .rates
        .into_keys()
        .map(|code| code.to_ascii_lowercase())
        .collect::<Vec<_>>();

    supported_currencies.extend(
        DEFAULT_STABLECOIN_SOURCES
            .iter()
            .map(|(code, _)| (*code).to_owned()),
    );

    supported_currencies.sort_unstable();
    supported_currencies.dedup();

    Ok(SupportedMarketCurrenciesResponse {
        supported_currencies,
    })
}

async fn fetch_market_rate(
    state: &AppState,
    payment_token: &PaymentTokenMetadata,
    market_currency: String,
) -> Result<MarketRate, AuthError> {
    let payment_token_usd_quote = fetch_payment_token_usd_quote(state, payment_token).await?;
    let payment_token_symbol = payment_token.symbol.to_ascii_lowercase();

    if market_currency == payment_token_symbol {
        return Ok(MarketRate {
            market_currency,
            market_currency_per_payment_token: Decimal::ONE,
            usd_per_payment_token: payment_token_usd_quote.usd_per_payment_token,
            last_updated_at: payment_token_usd_quote.last_updated_at,
            payment_token_coin_id: payment_token_usd_quote.coin_id,
        });
    }

    let market_currency_per_payment_token = match resolve_market_currency_kind(&market_currency) {
        MarketCurrencyKind::Fiat { code_lower, .. } if code_lower == "usd" => {
            payment_token_usd_quote.usd_per_payment_token
        }
        MarketCurrencyKind::Fiat { code_upper, .. } => {
            let market_currency_per_usd = fetch_fiat_rate_against_usd(state, &code_upper).await?;
            round_decimal(
                market_currency_per_usd * payment_token_usd_quote.usd_per_payment_token,
                12,
            )
        }
        MarketCurrencyKind::Stablecoin {
            code_lower,
            coin_id,
        } => {
            let usd_per_market_currency =
                fetch_source_stablecoin_usd_price(state, &code_lower, coin_id).await?;
            round_decimal(
                payment_token_usd_quote.usd_per_payment_token / usd_per_market_currency,
                12,
            )
        }
    };

    if market_currency_per_payment_token <= Decimal::ZERO {
        return Err(AuthError::internal(
            "computed market rate must be positive",
            format!(
                "market_currency={} market_currency_per_payment_token={}",
                market_currency, market_currency_per_payment_token
            ),
        ));
    }

    Ok(MarketRate {
        market_currency,
        market_currency_per_payment_token,
        usd_per_payment_token: payment_token_usd_quote.usd_per_payment_token,
        last_updated_at: payment_token_usd_quote.last_updated_at,
        payment_token_coin_id: payment_token_usd_quote.coin_id,
    })
}

async fn fetch_fiat_rate_against_usd(
    state: &AppState,
    quote_currency: &str,
) -> Result<Decimal, AuthError> {
    let url = format!(
        "{}/latest/USD",
        state.env.exchange_rate_api_base_url.trim_end_matches('/'),
    );

    let response = exchange_rate_api_request(state.http_client.get(url)).await?;
    let payload = response
        .json::<ExchangeRateApiLatestResponse>()
        .await
        .map_err(|error| {
            AuthError::internal("failed to decode ExchangeRate-API latest response", error)
        })?;

    ensure_exchange_rate_api_success(&payload)?;

    let number = payload.rates.get(quote_currency).ok_or_else(|| {
        AuthError::bad_request(format!(
            "unsupported market_currency `{}`",
            quote_currency.to_ascii_lowercase()
        ))
    })?;

    decimal_from_number(number, "Frankfurter rate")
}

async fn fetch_payment_token_usd_quote(
    state: &AppState,
    payment_token: &PaymentTokenMetadata,
) -> Result<PaymentTokenUsdQuote, AuthError> {
    let coin_id = state
        .env
        .coinpaprika_payment_token_coin_id
        .trim()
        .to_owned();
    match fetch_coinpaprika_usd_quote(state, &coin_id).await {
        Ok((usd_per_payment_token, last_updated_at)) => Ok(PaymentTokenUsdQuote {
            coin_id,
            usd_per_payment_token,
            last_updated_at,
        }),
        Err(error) if is_usd_stable_symbol(&payment_token.symbol) => {
            tracing::warn!(
                payment_token_symbol = %payment_token.symbol,
                ?error,
                "falling back to 1 USD parity for payment token",
            );
            Ok(PaymentTokenUsdQuote {
                coin_id,
                usd_per_payment_token: Decimal::ONE,
                last_updated_at: None,
            })
        }
        Err(error) => Err(error),
    }
}

async fn fetch_source_stablecoin_usd_price(
    state: &AppState,
    code_lower: &str,
    coin_id: &str,
) -> Result<Decimal, AuthError> {
    match fetch_coinpaprika_usd_quote(state, coin_id).await {
        Ok((usd_per_market_currency, _)) => Ok(usd_per_market_currency),
        Err(error) if is_usd_stable_code(code_lower) => {
            tracing::warn!(
                market_currency = %code_lower,
                ?error,
                "falling back to 1 USD parity for source stablecoin",
            );
            Ok(Decimal::ONE)
        }
        Err(error) => Err(error),
    }
}

async fn fetch_coinpaprika_usd_quote(
    state: &AppState,
    coin_id: &str,
) -> Result<(Decimal, Option<i64>), AuthError> {
    let url = format!(
        "{}/tickers/{}",
        state.env.coinpaprika_api_base_url.trim_end_matches('/'),
        coin_id,
    );

    let response =
        coinpaprika_request(state.http_client.get(url).query(&[("quotes", "USD")])).await?;
    let payload = response
        .json::<CoinPaprikaTickerResponse>()
        .await
        .map_err(|error| {
            AuthError::internal("failed to decode CoinPaprika ticker response", error)
        })?;

    let quote = payload.quotes.get("USD").ok_or_else(|| {
        AuthError::service_unavailable("CoinPaprika ticker response is missing USD quote data")
    })?;
    let usd_per_coin = decimal_from_number(&quote.price, "CoinPaprika USD price")?;

    if usd_per_coin <= Decimal::ZERO {
        return Err(AuthError::service_unavailable(
            "CoinPaprika returned a non-positive USD price",
        ));
    }

    let last_updated_at = payload.last_updated.as_deref().and_then(parse_timestamp);

    Ok((usd_per_coin, last_updated_at))
}

async fn read_payment_token_metadata(env: &Environment) -> Result<PaymentTokenMetadata, AuthError> {
    let token_address = parse_contract_address(&env.payment_token_address)
        .map_err(|error| AuthError::internal("invalid payment token address", error))?;
    let contract = read_erc20_contract(env, token_address)
        .await
        .map_err(|error| {
            AuthError::internal("failed to build payment token read contract", error)
        })?;

    let symbol = contract
        .method::<_, String>("symbol", ())
        .map_err(|error| AuthError::internal("failed to build payment token symbol call", error))?
        .call()
        .await
        .map_err(|error| AuthError::internal("failed to call payment token symbol", error))?;
    let decimals = contract
        .method::<_, u8>("decimals", ())
        .map_err(|error| AuthError::internal("failed to build payment token decimals call", error))?
        .call()
        .await
        .map_err(|error| AuthError::internal("failed to call payment token decimals", error))?;

    Ok(PaymentTokenMetadata {
        address: format_address(token_address),
        symbol,
        decimals,
    })
}

async fn read_erc20_contract(
    env: &Environment,
    token_address: Address,
) -> Result<Contract<Provider<Http>>> {
    let provider = rpc::monad_provider_arc(env).await?;
    Ok(Contract::new(token_address, erc20_abi()?, provider))
}

async fn exchange_rate_api_request(
    request: RequestBuilder,
) -> Result<reqwest::Response, AuthError> {
    let response = request
        .send()
        .await
        .map_err(|error| AuthError::internal("failed to reach ExchangeRate-API", error))?;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        return Err(AuthError::too_many_requests(
            "ExchangeRate-API rate limit exceeded, retry shortly",
        ));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!(%status, body, "exchange rate api request failed");
        return Err(AuthError::service_unavailable(
            "ExchangeRate-API request failed, currency service is temporarily unavailable",
        ));
    }

    Ok(response)
}

fn ensure_exchange_rate_api_success(
    payload: &ExchangeRateApiLatestResponse,
) -> Result<(), AuthError> {
    match payload.result.as_deref() {
        None | Some("success") => Ok(()),
        Some(result) => Err(AuthError::service_unavailable(format!(
            "ExchangeRate-API returned unexpected result `{result}`",
        ))),
    }
}

async fn coinpaprika_request(request: RequestBuilder) -> Result<reqwest::Response, AuthError> {
    let response = request
        .send()
        .await
        .map_err(|error| AuthError::internal("failed to reach CoinPaprika", error))?;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        return Err(AuthError::too_many_requests(
            "CoinPaprika rate limit exceeded, retry shortly",
        ));
    }

    if response.status() == StatusCode::NOT_FOUND {
        return Err(AuthError::bad_request(
            "unsupported payment token or stablecoin market source",
        ));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!(%status, body, "coinpaprika request failed");
        return Err(AuthError::service_unavailable(
            "CoinPaprika request failed, crypto pricing is temporarily unavailable",
        ));
    }

    Ok(response)
}

fn build_amount_quote(
    raw_market_amount: Option<&str>,
    market_currency_per_payment_token: Decimal,
    payment_token_decimals: u8,
    field_name: &str,
) -> Result<Option<MarketAmountQuote>, AuthError> {
    let Some(raw_market_amount) = raw_market_amount
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    let market_amount = Decimal::from_str(raw_market_amount)
        .map_err(|_| AuthError::bad_request(format!("invalid {field_name} decimal amount")))?;
    if market_amount < Decimal::ZERO {
        return Err(AuthError::bad_request(format!(
            "{field_name} must be greater than or equal to zero",
        )));
    }

    let payment_token_amount = round_decimal(
        market_amount / market_currency_per_payment_token,
        payment_token_decimals.into(),
    );
    let payment_token_base_units =
        decimal_to_base_units(payment_token_amount, payment_token_decimals)?;

    Ok(Some(MarketAmountQuote {
        market_currency_amount: format_decimal(market_amount),
        payment_token_amount: format_decimal(payment_token_amount),
        payment_token_base_units,
    }))
}

fn resolve_market_currency_kind(raw: &str) -> MarketCurrencyKind {
    if let Some((_, coin_id)) = DEFAULT_STABLECOIN_SOURCES
        .iter()
        .find(|(code, _)| *code == raw)
    {
        return MarketCurrencyKind::Stablecoin {
            code_lower: raw.to_owned(),
            coin_id,
        };
    }

    MarketCurrencyKind::Fiat {
        code_lower: raw.to_owned(),
        code_upper: raw.to_ascii_uppercase(),
    }
}

fn is_usd_stable_symbol(symbol: &str) -> bool {
    is_usd_stable_code(&symbol.to_ascii_lowercase())
}

fn is_usd_stable_code(code: &str) -> bool {
    matches!(code, "usd" | "usdc" | "usdt")
}

fn normalize_market_currency(raw: &str) -> Result<String, AuthError> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(AuthError::bad_request("market_currency is required"));
    }
    if !normalized
        .chars()
        .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    {
        return Err(AuthError::bad_request(
            "market_currency must be lowercase letters/numbers only",
        ));
    }

    Ok(normalized)
}

fn decimal_from_number(number: &Number, context: &'static str) -> Result<Decimal, AuthError> {
    Decimal::from_str(&number.to_string()).map_err(|error| AuthError::internal(context, error))
}

fn parse_timestamp(raw: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.timestamp())
        .ok()
}

fn decimal_to_base_units(value: Decimal, decimals: u8) -> Result<String, AuthError> {
    if decimals > 28 {
        return Err(AuthError::internal(
            "unsupported payment token decimals",
            format!("decimals {decimals} exceeds supported precision"),
        ));
    }

    let multiplier = decimal_power_of_ten(decimals);
    let scaled = round_decimal(value * multiplier, 0);
    let normalized = scaled.normalize().to_string();

    if normalized.contains('.') {
        return Err(AuthError::internal(
            "failed to derive integer payment token base units",
            format!("non-integer base unit value {normalized}"),
        ));
    }

    Ok(normalized)
}

fn decimal_power_of_ten(decimals: u8) -> Decimal {
    let mut value = Decimal::ONE;
    for _ in 0..decimals {
        value *= Decimal::TEN;
    }
    value
}

fn round_decimal(value: Decimal, scale: u32) -> Decimal {
    value.round_dp_with_strategy(scale, RoundingStrategy::MidpointAwayFromZero)
}

fn format_decimal(value: Decimal) -> String {
    value.normalize().to_string()
}
