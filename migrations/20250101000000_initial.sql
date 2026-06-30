CREATE TABLE IF NOT EXISTS guild_configs (
    guild_id TEXT PRIMARY KEY NOT NULL,
    moderation_enabled INTEGER NOT NULL DEFAULT 0,
    auto_mod_enabled INTEGER NOT NULL DEFAULT 0,
    logging_enabled INTEGER NOT NULL DEFAULT 0,
    auto_mod_config TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS warnings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    moderator_id TEXT NOT NULL,
    reason TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_warnings_lookup ON warnings(guild_id, user_id);
