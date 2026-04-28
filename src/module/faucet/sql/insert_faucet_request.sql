INSERT INTO faucet_requests (
    id,
    user_id,
    wallet_address,
    token_address,
    amount,
    tx_hash
) VALUES (
    $1,
    $2,
    $3,
    $4,
    $5,
    $6
)
RETURNING
    id,
    user_id,
    wallet_address,
    token_address,
    amount,
    tx_hash,
    requested_at;
