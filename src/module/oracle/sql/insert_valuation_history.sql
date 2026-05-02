INSERT INTO oracle_valuation_history (
    asset_address,
    asset_value,
    nav_per_token,
    onchain_updated_at,
    reference_id,
    tx_hash,
    updated_by_user_id,
    observed_at
) VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8
)
ON CONFLICT (asset_address, onchain_updated_at, reference_id) DO NOTHING
