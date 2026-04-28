SELECT
    id,
    user_id,
    wallet_address,
    token_address,
    amount,
    tx_hash,
    requested_at
FROM faucet_requests
WHERE user_id = $1
ORDER BY requested_at DESC
LIMIT 1;
