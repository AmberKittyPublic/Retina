CREATE TABLE IF NOT EXISTS giveaways (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    prize TEXT NOT NULL,
    winners_count INTEGER NOT NULL DEFAULT 1,
    end_time TEXT NOT NULL,
    ended INTEGER NOT NULL DEFAULT 0,
    entries TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(guild_id, message_id)
);
