ALTER TABLE assets
ADD COLUMN IF NOT EXISTS chain_id BIGINT;

UPDATE assets
SET chain_id = 0
WHERE chain_id IS NULL;

ALTER TABLE assets
ALTER COLUMN chain_id SET NOT NULL;

ALTER TABLE assets
ALTER COLUMN chain_id SET DEFAULT 10143;

ALTER TABLE assets
DROP CONSTRAINT IF EXISTS assets_proposal_id_key;

CREATE UNIQUE INDEX IF NOT EXISTS assets_chain_id_proposal_id_key
ON assets (chain_id, proposal_id);

CREATE INDEX IF NOT EXISTS assets_chain_id_idx
ON assets (chain_id);
