CREATE TABLE IF NOT EXISTS admin_upload_assets (
    id UUID PRIMARY KEY,
    storage_provider TEXT NOT NULL,
    bucket_name TEXT NOT NULL,
    scope TEXT NOT NULL,
    file_name TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    cid TEXT NOT NULL,
    ipfs_url TEXT NOT NULL,
    gateway_url TEXT NOT NULL,
    created_by_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS admin_upload_assets_created_by_user_id_idx
ON admin_upload_assets (created_by_user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS asset_catalog_entries (
    asset_address TEXT PRIMARY KEY REFERENCES assets(asset_address) ON DELETE CASCADE,
    slug TEXT UNIQUE NOT NULL,
    image_url TEXT,
    summary TEXT,
    featured BOOLEAN NOT NULL DEFAULT FALSE,
    visible BOOLEAN NOT NULL DEFAULT TRUE,
    searchable BOOLEAN NOT NULL DEFAULT TRUE,
    created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS asset_catalog_entries_featured_idx
ON asset_catalog_entries (featured, updated_at DESC);

CREATE INDEX IF NOT EXISTS asset_catalog_entries_visible_idx
ON asset_catalog_entries (visible, updated_at DESC);
