CREATE TABLE IF NOT EXISTS xp_config (
    guild_id TEXT PRIMARY KEY NOT NULL,
    xp_per_message INTEGER NOT NULL DEFAULT 20,
    cooldown_seconds INTEGER NOT NULL DEFAULT 60,
    min_chars INTEGER NOT NULL DEFAULT 1,
    voice_xp_enabled INTEGER NOT NULL DEFAULT 0,
    voice_xp_interval_minutes INTEGER NOT NULL DEFAULT 5
);

CREATE TABLE IF NOT EXISTS xp_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    xp INTEGER NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 1,
    last_xp_time TEXT,
    UNIQUE(guild_id, user_id)
);

CREATE TABLE IF NOT EXISTS xp_rewards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    level INTEGER NOT NULL,
    role_id TEXT NOT NULL,
    UNIQUE(guild_id, role_id)
);
