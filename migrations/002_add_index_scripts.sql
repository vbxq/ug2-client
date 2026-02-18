ALTER TABLE discord_builds
    ADD COLUMN IF NOT EXISTS index_scripts JSONB NOT NULL DEFAULT '[]'::jsonb;
