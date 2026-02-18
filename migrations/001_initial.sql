CREATE TABLE IF NOT EXISTS discord_builds (
    id SERIAL PRIMARY KEY,
    build_hash VARCHAR(64) NOT NULL UNIQUE,
    channel VARCHAR(16) NOT NULL DEFAULT 'canary',
    build_date TIMESTAMPTZ NOT NULL,
    global_env JSONB,
    scripts JSONB NOT NULL,
    index_scripts JSONB NOT NULL DEFAULT '[]'::jsonb,
    is_patched BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_builds_hash ON discord_builds(build_hash);
CREATE INDEX IF NOT EXISTS idx_builds_channel ON discord_builds(channel);
CREATE INDEX IF NOT EXISTS idx_builds_active ON discord_builds(is_active) WHERE is_active = TRUE;

CREATE TABLE IF NOT EXISTS asset_cache (
    id SERIAL PRIMARY KEY,
    build_hash VARCHAR(64) NOT NULL REFERENCES discord_builds(build_hash),
    asset_name VARCHAR(256) NOT NULL,
    content_type VARCHAR(64) NOT NULL DEFAULT 'application/javascript',
    file_size BIGINT NOT NULL DEFAULT 0,
    is_patched BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_accessed TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(build_hash, asset_name)
);

CREATE INDEX IF NOT EXISTS idx_assets_build ON asset_cache(build_hash);
CREATE INDEX IF NOT EXISTS idx_assets_name ON asset_cache(asset_name);
