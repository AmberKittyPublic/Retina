CREATE TABLE IF NOT EXISTS scheduled_announcements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT 'Announcement',
    message TEXT NOT NULL,
    interval_minutes INTEGER,
    next_run_at TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT NOT NULL
);
