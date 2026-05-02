SELECT
    asset_address,
    asset_value,
    nav_per_token,
    onchain_updated_at,
    reference_id,
    tx_hash,
    updated_by_user_id,
    observed_at,
    created_at
FROM oracle_valuation_history
WHERE asset_address = $1
  AND ($2::timestamptz IS NULL OR observed_at >= $2)
ORDER BY observed_at ASC
