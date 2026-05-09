CREATE TABLE IF NOT EXISTS ticket_config (
    guild_id TEXT PRIMARY KEY NOT NULL,
    category_id TEXT NOT NULL DEFAULT '',
    staff_role_id TEXT NOT NULL DEFAULT '',
    panel_channel_id TEXT NOT NULL DEFAULT '',
    panel_message_id TEXT NOT NULL DEFAULT '',
    enabled INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tickets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    creator_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    closed_at TEXT,
    UNIQUE(guild_id, channel_id)
);
