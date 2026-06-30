ALTER TABLE guild_configs ADD COLUMN welcome_enabled INTEGER NOT NULL DEFAULT 0;
ALTER TABLE guild_configs ADD COLUMN welcome_config TEXT NOT NULL DEFAULT '{}';
