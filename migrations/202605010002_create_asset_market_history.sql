CREATE TABLE IF NOT EXISTS asset_price_history (
    id BIGSERIAL PRIMARY KEY,
    asset_address TEXT NOT NULL REFERENCES assets(asset_address) ON DELETE CASCADE,
    price_per_token TEXT NOT NULL,
    redemption_price_per_token TEXT NOT NULL,
    source TEXT NOT NULL,
    tx_hash TEXT,
    created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    observed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS asset_price_history_asset_address_observed_at_idx
ON asset_price_history (asset_address, observed_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS asset_price_history_asset_address_tx_hash_uidx
ON asset_price_history (asset_address, tx_hash)
WHERE tx_hash IS NOT NULL;

CREATE TABLE IF NOT EXISTS oracle_valuation_history (
    id BIGSERIAL PRIMARY KEY,
    asset_address TEXT NOT NULL REFERENCES assets(asset_address) ON DELETE CASCADE,
    asset_value TEXT NOT NULL,
    nav_per_token TEXT NOT NULL,
    onchain_updated_at BIGINT NOT NULL,
    reference_id TEXT NOT NULL,
    tx_hash TEXT,
    updated_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    observed_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS oracle_valuation_history_asset_address_observed_at_idx
ON oracle_valuation_history (asset_address, observed_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS oracle_valuation_history_asset_address_updated_ref_uidx
ON oracle_valuation_history (asset_address, onchain_updated_at, reference_id);

INSERT INTO asset_price_history (
    asset_address,
    price_per_token,
    redemption_price_per_token,
    source,
    tx_hash,
    created_by_user_id,
    observed_at
)
SELECT
    asset_address,
    price_per_token,
    redemption_price_per_token,
    'bootstrap',
    last_tx_hash,
    updated_by_user_id,
    updated_at
FROM assets
ON CONFLICT (asset_address, tx_hash) WHERE tx_hash IS NOT NULL DO NOTHING;

INSERT INTO oracle_valuation_history (
    asset_address,
    asset_value,
    nav_per_token,
    onchain_updated_at,
    reference_id,
    tx_hash,
    updated_by_user_id,
    observed_at
)
SELECT
    asset_address,
    asset_value,
    nav_per_token,
    onchain_updated_at,
    reference_id,
    last_tx_hash,
    updated_by_user_id,
    to_timestamp(onchain_updated_at)
FROM oracle_valuations
ON CONFLICT (asset_address, onchain_updated_at, reference_id) DO NOTHING;
