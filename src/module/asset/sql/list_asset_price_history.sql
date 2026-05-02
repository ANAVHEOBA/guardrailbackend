SELECT
    asset_address,
    price_per_token,
    redemption_price_per_token,
    source,
    tx_hash,
    created_by_user_id,
    observed_at,
    created_at
FROM asset_price_history
WHERE asset_address = $1
  AND ($2::timestamptz IS NULL OR observed_at >= $2)
ORDER BY observed_at ASC
