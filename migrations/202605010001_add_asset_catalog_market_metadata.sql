ALTER TABLE asset_catalog_entries
ADD COLUMN IF NOT EXISTS market_segment TEXT;

ALTER TABLE asset_catalog_entries
ADD COLUMN IF NOT EXISTS suggested_internal_tags TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];

ALTER TABLE asset_catalog_entries
ADD COLUMN IF NOT EXISTS sources TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];
