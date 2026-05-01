# RWA Admin Asset Seeds

Updated: 2026-05-01

This file is a working seed list for creating RWA assets through this backend's admin flow.

It combines:

1. The exact fields your backend expects.
2. Research-backed asset ideas from Ondo, Centrifuge, and RWA.xyz market context.
3. Suggested default values where the source platform does not publish a direct backend-ready value.

## 1. Admin Flow In This Repo

### A. Register the asset type first

Endpoint: `POST /assets/types`

Fields:

- `asset_type_id`
- `asset_type_name`
- `implementation_address`

Notes:

- `asset_type_id` is parsed as `bytes32`, but this backend accepts plain text up to 32 bytes, so values like `US_TREASURY` or `PRIVATE_CREDIT` are valid inputs.
- Register this once per product category, not once per individual asset.

Suggested starter asset types:

| `asset_type_id` | `asset_type_name` |
| --- | --- |
| `US_TREASURY` | U.S. Treasury Fund |
| `YIELD_NOTE` | Yield-Bearing Note |
| `PRIVATE_CREDIT` | Private Credit Fund |
| `INDEX_FUND` | Tokenized Index Fund |
| `TOKENIZED_STOCK` | Tokenized Stock |

### B. Upload an image if you want catalog artwork

Endpoint: `POST /uploads/images`

Notes:

- The upload response includes both `ipfs_url` and `gateway_url`.
- Use `gateway_url` as the easiest `image_url` for the asset catalog payload.

### C. Create the asset

Endpoint: `POST /assets`

This backend mixes:

- onchain creation fields
- catalog/display fields

into the same payload.

### D. Optional follow-up endpoints after asset creation

- `PUT /assets/{asset_address}/catalog`
- `PUT /assets/{asset_address}/metadata`
- `PUT /assets/{asset_address}/pricing`
- `PUT /assets/{asset_address}/compliance-registry`
- `PUT /assets/{asset_address}/treasury`
- `PUT /compliance/assets/{asset}/rules`
- `PUT /compliance/assets/{asset}/jurisdictions/{jurisdiction}`
- `POST /oracle/valuations`
- `POST /oracle/valuations/sync-pricing`

Use `sync-pricing` when the asset price is NAV-based and should track an external valuation source.

## 2. Exact Create-Asset Fields

Endpoint: `POST /assets`

| Field | Required | Backend behavior | Practical guidance |
| --- | --- | --- | --- |
| `proposal_id` | yes | parsed as `uint256` string | internal unique ID per chain |
| `asset_type_id` | yes | parsed as `bytes32` | must already be registered |
| `name` | yes | plain string | issuer/product display name |
| `symbol` | yes | plain string | token ticker |
| `max_supply` | yes | parsed as `uint256` string | choose cap in token base units |
| `subscription_price` | yes | parsed as `uint256` string | initial purchase price |
| `redemption_price` | yes | parsed as `uint256` string | initial redemption price |
| `self_service_purchase_enabled` | yes | boolean | keep `false` until compliance and liquidity are ready unless the product should be user-purchasable immediately |
| `metadata_hash` | optional | parsed as `bytes32`; if omitted it becomes zero-bytes | use a real hash later if you anchor docs/metadata |
| `slug` | optional | normalized to lowercase hyphen slug; if omitted it is generated from `name` | keep short and stable |
| `image_url` | optional | blank becomes `null` | use uploaded `gateway_url` or issuer-hosted image |
| `summary` | optional | blank becomes `null` | short catalog description |
| `featured` | optional | default `false` | homepage or highlighted asset |
| `visible` | optional | default `true` | hide only if internal or draft |
| `searchable` | optional | default `true` | turn off only if hidden from search |

Important implementation notes:

- All large numeric fields are strings, not JSON numbers.
- The repo does not document the pricing decimal convention. Treat `subscription_price`, `redemption_price`, `asset_value`, and `nav_per_token` as implementation-specific `uint256` values and keep them consistent with the asset contract/oracle you deploy.
- Slugs accept letters and numbers and normalize separators into hyphens.

## 2.1 Related Follow-Up Payloads

### Register asset type

Endpoint: `POST /assets/types`

| Field | Required | Notes |
| --- | --- | --- |
| `asset_type_id` | yes | `bytes32` input, plain text up to 32 bytes is accepted |
| `asset_type_name` | yes | display label |
| `implementation_address` | yes | deployed implementation contract |

### Update catalog later

Endpoint: `PUT /assets/{asset_address}/catalog`

| Field | Required | Notes |
| --- | --- | --- |
| `slug` | yes | normalized lowercase hyphen slug |
| `image_url` | no | uploaded `gateway_url` works well |
| `summary` | no | short catalog description |
| `featured` | no | default `false` |
| `visible` | no | default `true` |
| `searchable` | no | default `true` |

### Set compliance rules

Endpoint: `PUT /compliance/assets/{asset}/rules`

| Field | Required | Notes |
| --- | --- | --- |
| `transfers_enabled` | yes | global transfer toggle |
| `subscriptions_enabled` | yes | global purchase/subscription toggle |
| `redemptions_enabled` | yes | global redemption toggle |
| `requires_accreditation` | yes | useful for institutional products |
| `min_investment` | yes | `uint256` string |
| `max_investor_balance` | yes | `uint256` string |

### Submit valuation and sync pricing

Endpoint: `POST /oracle/valuations/sync-pricing`

| Field | Required | Notes |
| --- | --- | --- |
| `asset_address` | yes | created asset address |
| `asset_value` | yes | `uint256` string |
| `nav_per_token` | yes | `uint256` string |
| `subscription_price` | yes | `uint256` string |
| `redemption_price` | yes | `uint256` string |
| `reference_id` | yes | source/version marker, parsed as text/bytes32 style input |

## 3. Market Context From RWA.xyz

As of 2026-05-01 on RWA.xyz tokenization platform rankings:

- `Securitize` is listed `#1` by distributed RWA value.
- `Ondo` is listed `#2` with `265` RWAs and about `$3.61B` distributed RWA value.
- `Centrifuge` is listed `#7` with `8` RWAs and about `$1.76B` distributed RWA value.

Why these platforms are good seed candidates:

- Ondo is one of the largest live issuance/distribution platforms in the dataset.
- Centrifuge spans treasury, credit, and equity-index style products.
- RWA.xyz is useful for deciding which product categories deserve first-class support in your admin panel.

## 4. Asset Drafts

The blocks below are meant as admin seeds.

What is source-backed:

- `name`
- `symbol`
- asset class
- general product description
- likely compliance posture

What is still internal to your system:

- `proposal_id`
- `max_supply`
- actual `subscription_price`
- actual `redemption_price`
- final `metadata_hash`
- final `image_url`

### 4.1 Ondo OUSG

Platform: Ondo

Asset class: U.S. Treasuries

Why it fits:

- Ondo documents OUSG as a tokenized short-term U.S. government treasury product.
- It is NAV-based and 24/7 mintable/redeemable for eligible investors.
- It is a strong template for an institutional cash-management asset.

Suggested admin seed:

```yaml
proposal_id: "10001"
asset_type_id: "US_TREASURY"
name: "Ondo Short-Term US Government Treasuries"
symbol: "OUSG"
max_supply: "<set cap in base units>"
subscription_price: "<set current NAV-scaled price>"
redemption_price: "<set current NAV-scaled price>"
self_service_purchase_enabled: false
metadata_hash: null
slug: "ondo-ousg"
image_url: "<upload image and use gateway_url>"
summary: "Tokenized short-term U.S. Treasuries for eligible investors with NAV-based pricing and 24/7 minting and redemption."
featured: true
visible: true
searchable: true
```

Suggested follow-up profile:

- compliance posture: qualified-access / accredited / permissioned
- `requires_accreditation`: `true`
- pricing mode: NAV-based
- good candidate for `POST /oracle/valuations/sync-pricing`

### 4.2 Ondo USDY

Platform: Ondo

Asset class: Yield-bearing note backed by treasury-linked assets / cash equivalents

Why it fits:

- Ondo documents USDY as a tokenized note designed for qualifying non-US users.
- The accumulating form uses a rising redemption value over time, which maps well to your separate subscription and redemption price fields.
- It is a good template for a yield note rather than a fund share.

Suggested admin seed:

```yaml
proposal_id: "10002"
asset_type_id: "YIELD_NOTE"
name: "Ondo USD Yield"
symbol: "USDY"
max_supply: "<set cap in base units>"
subscription_price: "<set current reference price>"
redemption_price: "<set current reference price>"
self_service_purchase_enabled: false
metadata_hash: null
slug: "ondo-usdy"
image_url: "<upload image and use gateway_url>"
summary: "Tokenized yield-bearing dollar note backed by short-term U.S. Treasury-linked assets and cash-equivalent exposure."
featured: true
visible: true
searchable: true
```

Suggested follow-up profile:

- compliance posture: non-US eligible investors only
- `requires_accreditation`: depends on your own product design; Ondo's source docs frame it around qualified non-US access rather than a generic retail product
- pricing mode: reference-price / yield-accreting

### 4.3 Ondo GM Tesla Example

Platform: Ondo Global Markets

Asset class: Tokenized stock / total-return tracker

Why it fits:

- Ondo Global Markets documents 100+ tokenized stocks and ETFs.
- The docs explicitly use Tesla as the naming example and state that `TSLA` becomes `TSLAon`.
- This is the cleanest seed if you want to model tokenized public equities in your admin catalog.

Suggested admin seed:

```yaml
proposal_id: "10003"
asset_type_id: "TOKENIZED_STOCK"
name: "Ondo Tesla Total Return Tracker"
symbol: "TSLAon"
max_supply: "<set cap in base units>"
subscription_price: "<set current reference price>"
redemption_price: "<set current reference price>"
self_service_purchase_enabled: false
metadata_hash: null
slug: "ondo-tslaon"
image_url: "<upload image and use gateway_url>"
summary: "Tokenized total-return exposure to Tesla through Ondo Global Markets."
featured: false
visible: true
searchable: true
```

Inference note:

- The exact display name above is a practical seed, not a quoted issuer product name.
- The `TSLAon` symbol is directly consistent with Ondo's documented `on` suffix naming convention.

Suggested follow-up profile:

- compliance posture: currently institutional onboarding only per Ondo GM docs
- asset type: best kept separate from treasury and credit products
- pricing mode: market-driven / reference-price based

### 4.4 Centrifuge JTRSY

Platform: Centrifuge / Janus Henderson Anemoy

Asset class: U.S. Treasury fund

Why it fits:

- Centrifuge describes JTRSY as onchain exposure to short-duration U.S. Treasury Bills.
- It is a live institutional-grade treasury product with ratings and daily liquidity.
- It is one of the strongest treasury comparables to OUSG.

Suggested admin seed:

```yaml
proposal_id: "10004"
asset_type_id: "US_TREASURY"
name: "Janus Henderson Anemoy Treasury Fund"
symbol: "JTRSY"
max_supply: "<set cap in base units>"
subscription_price: "<set current NAV-scaled price>"
redemption_price: "<set current NAV-scaled price>"
self_service_purchase_enabled: false
metadata_hash: null
slug: "centrifuge-jtrsy"
image_url: "<upload image and use gateway_url>"
summary: "Tokenized short-duration U.S. Treasury bill exposure powered by Centrifuge and managed within the Janus Henderson Anemoy structure."
featured: true
visible: true
searchable: true
```

Suggested follow-up profile:

- compliance posture: institutional / permissioned
- `requires_accreditation`: `true`
- pricing mode: NAV-based

### 4.5 Centrifuge JAAA

Platform: Centrifuge / Janus Henderson Anemoy

Asset class: Structured credit / AAA CLO fund

Why it fits:

- Centrifuge describes JAAA as the first AAA-rated CLO fund brought fully onchain.
- It is a clean private-credit seed that differs materially from treasury products.
- Good choice if you want an admin example for corporate credit rather than sovereign debt.

Suggested admin seed:

```yaml
proposal_id: "10005"
asset_type_id: "PRIVATE_CREDIT"
name: "Janus Henderson Anemoy AAA CLO Fund"
symbol: "JAAA"
max_supply: "<set cap in base units>"
subscription_price: "<set current NAV-scaled price>"
redemption_price: "<set current NAV-scaled price>"
self_service_purchase_enabled: false
metadata_hash: null
slug: "centrifuge-jaaa"
image_url: "<upload image and use gateway_url>"
summary: "Tokenized AAA-rated CLO fund with daily liquidity and institutional-grade credit exposure."
featured: true
visible: true
searchable: true
```

Suggested follow-up profile:

- compliance posture: institutional / permissioned
- `requires_accreditation`: `true`
- pricing mode: NAV-based

### 4.6 Centrifuge deSPXA

Platform: Centrifuge

Asset class: Tokenized equity index exposure

Why it fits:

- Centrifuge describes deSPXA as tokenized exposure to the Anemoy S&P 500 fund.
- It is designed for eligible non-US users and intended to be usable inside DeFi.
- It gives you a very different admin seed from treasury and credit, while still clearly fitting the RWA category.

Suggested admin seed:

```yaml
proposal_id: "10006"
asset_type_id: "INDEX_FUND"
name: "deSPXA"
symbol: "deSPXA"
max_supply: "<set cap in base units>"
subscription_price: "<set current NAV-scaled price>"
redemption_price: "<set current NAV-scaled price>"
self_service_purchase_enabled: false
metadata_hash: null
slug: "centrifuge-despxa"
image_url: "<upload image and use gateway_url>"
summary: "Tokenized S&P 500 fund exposure for eligible non-US users, designed for 24/7 onchain utility."
featured: false
visible: true
searchable: true
```

Suggested follow-up profile:

- compliance posture: non-US eligible distribution
- `requires_accreditation`: depends on your legal structure, not just the market category
- pricing mode: NAV-based

## 5. Practical Defaults I Would Use

If the goal is to launch cleanly and avoid accidental public purchasing before setup is finished:

- set `self_service_purchase_enabled` to `false` on creation
- set `visible` to `true`
- set `searchable` to `true`
- set `featured` only for 1-3 flagship assets
- upload catalog images first, then use the returned `gateway_url`
- set initial pricing once, then move NAV-style products onto the oracle flow

## 6. Sources

Repo implementation basis:

- `src/module/asset/schema.rs`
- `src/service/asset/mod.rs`
- `src/module/asset/route.rs`
- `src/module/admin/route.rs`
- `src/service/upload.rs`
- `src/module/compliance/schema.rs`
- `src/module/oracle/schema.rs`

External sources used:

- RWA.xyz platform rankings: https://app.rwa.xyz/platforms
- RWA.xyz treasury market page: https://app.rwa.xyz/treasuries
- RWA.xyz credit market page: https://app.rwa.xyz/credit
- Ondo OUSG overview: https://docs.ondo.finance/qualified-access-products/ousg/overview
- Ondo OUSG eligibility: https://docs.ondo.finance/qualified-access-products/ousg/eligibility-and-onboarding
- Ondo USDY basics: https://docs.ondo.finance/general-access-products/usdy/basics
- Ondo Global Markets overview: https://docs.ondo.finance/ondo-global-markets/overview
- Ondo Global Markets available assets: https://docs.ondo.finance/ondo-global-markets/available-assets
- Centrifuge docs home: https://docs.centrifuge.io/
- Centrifuge deployments: https://docs.centrifuge.io/developer/protocol/deployments/
- Centrifuge JTRSY article: https://centrifuge.io/blog/jtrsy-aa-plus-rating
- Centrifuge JAAA article: https://centrifuge.io/blog/resolv-aave-centrifuge-partnership
- Centrifuge SPXA article: https://centrifuge.io/blog/centrifuge-launches-spxa
- Centrifuge deSPXA article: https://centrifuge.io/blog/despxa-on-base
