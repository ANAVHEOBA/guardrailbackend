INSERT INTO asset_price_history (
    asset_address,
    price_per_token,
    redemption_price_per_token,
    source,
    tx_hash,
    created_by_user_id,
    observed_at
) VALUES (
    $1, $2, $3, $4, $5, $6, COALESCE($7, NOW())
)
ON CONFLICT (asset_address, tx_hash) WHERE tx_hash IS NOT NULL DO NOTHING
