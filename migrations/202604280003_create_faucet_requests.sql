CREATE TABLE IF NOT EXISTS faucet_requests (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    wallet_address TEXT NOT NULL,
    token_address TEXT NOT NULL,
    amount TEXT NOT NULL,
    tx_hash TEXT NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS faucet_requests_user_id_requested_at_idx
ON faucet_requests (user_id, requested_at DESC);

CREATE INDEX IF NOT EXISTS faucet_requests_wallet_address_requested_at_idx
ON faucet_requests (wallet_address, requested_at DESC);
