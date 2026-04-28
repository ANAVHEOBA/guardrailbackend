INSERT INTO asset_catalog_entries (
    asset_address,
    slug,
    image_url,
    summary,
    featured,
    visible,
    searchable,
    created_by_user_id,
    updated_by_user_id
) VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8, $9
)
ON CONFLICT (asset_address) DO UPDATE
SET
    slug = EXCLUDED.slug,
    image_url = EXCLUDED.image_url,
    summary = EXCLUDED.summary,
    featured = EXCLUDED.featured,
    visible = EXCLUDED.visible,
    searchable = EXCLUDED.searchable,
    created_by_user_id = COALESCE(asset_catalog_entries.created_by_user_id, EXCLUDED.created_by_user_id),
    updated_by_user_id = EXCLUDED.updated_by_user_id,
    updated_at = NOW()
RETURNING
    asset_address,
    slug,
    image_url,
    summary,
    featured,
    visible,
    searchable,
    created_by_user_id,
    updated_by_user_id,
    created_at,
    updated_at
