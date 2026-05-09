# Retina — Discord Bot

A full-featured Discord bot written in Rust, aiming to be a feature-complete clone of **Dyno**. The bot and web dashboard run in the same process, sharing state via `Arc<RwLock<T>>`.

**Tech stack:** Rust, poise (slash commands), serenity (Discord API), axum (web server), sqlx + SQLite (database), tokio (async runtime).

## Features

### Moderation
`/ban` `/kick` `/warn` `/warnings` `/mute` `/purge` `/slowmode` `/lockdown` `/softban` `/members` `/move` `/voicekick` `/deafen` `/vmute` `/reason` `/case` `/notes` `/clearwarn` `/delwarn`

### Auto-moderation
7 rule types (spam, caps, links, emotes, mentions, banned words, max length) with 5 action types (delete, warn, timeout, kick, ban). Channel/role whitelist and blacklist support.

### Custom Commands
Lua-scripted commands via `!prefix`, with Discord API bindings.

### Reaction Roles
Emoji-to-role mapping on messages, automatic add/remove on reaction.

### Logging
Message edits/deletes, member join/leave, member updates, channel create/delete, voice state updates.

### Welcome / Goodbye
Customizable join/leave messages with `{user}`, `{mention}`, `{guild}` templates.

### Giveaways
Embed with reaction entry, random winner picker, scheduled draw.

### Tickets
Panel with 🎫 reaction, private channel creation, close/claim/add/remove, staff role.

### Leveling / XP
Message XP with cooldown, level-up role rewards, rank/leaderboard commands.

### Fun
24 commands: `rps`, `flip`, `roll`, `dadjoke`, `cat`, `dog`, `pug`, `github`, `urban`, `8ball`, `meme`, `number`, `roast`, `yomama`, `norris`, `pokemon`, `wouldyourather`, `space`, `translate`, `weather`, `remindme`, `timer`, `choose`, `poll`

### Misc
8 commands: `avatar`, `whois`, `serverinfo`, `membercount`, `randomcolor`, `invite`, `prefix`, `emotes`

### AFK
AFK status with mention detection and auto-remove on message.

### Self-Assignable Roles
`/addrank` `/delrank` `/rank` `/ranks` with join/leave toggle.

### Reminders
Background checker with `/remindme` command.

### Manager
`/addmod` `/delmod` `/listmods` `/nick` `/addrole` `/delrole`

### Scheduling
Tempban, tempmute, scheduled announcements with interval repeats.

### Web Dashboard
OAuth2 login, guild selector, per-server module toggles, auto-mod rule editor, welcome/goodbye config, custom command editor, reaction role manager, giveaway list, ticket management, XP config.

## Setup

1. **Create a Discord Bot**:
   - Go to https://discord.com/developers/applications
   - Create a new application
   - Go to "Bot" section, create a bot, enable "MESSAGE CONTENT INTENT"
   - Copy the bot token

2. **OAuth2 Setup**:
   - In "OAuth2" > "General", copy Client ID and Client Secret
   - Add redirect: `http://localhost:3000/auth/callback`

3. **Invite Bot**:
   - In "OAuth2" > "URL Generator", select "bot" and "applications.commands" scopes
   - Permissions: Administrator (or select specific ones)
   - Use the URL to invite the bot

4. **Configure**:
   ```bash
   cp config.example.toml config.toml
   # Edit config.toml with your token, client_id, client_secret
   ```

5. **Run**:
   ```bash
   cargo run
   ```

## Web Dashboard

Visit `http://localhost:3000` and log in with Discord to configure your servers.

## Project Structure

```
discord-bot/
├── build.rs                 # Compiles SCSS → CSS at build time
├── migrations/              # SQLite migrations (sqlx)
├── static/
│   ├── style.scss           # Dashboard styles
│   └── dashboard.js         # Client-side JS
├── templates/
│   ├── index.html           # Landing page
│   ├── dashboard.html       # Server list
│   ├── server.html          # Per-server config
│   └── partials/            # HTML fragments
├── src/
│   ├── main.rs              # Entrypoint
│   ├── cli.rs               # CLI arg parser
│   ├── types.rs             # AppState with shared state
│   ├── config/mod.rs        # Config structs
│   ├── database/mod.rs      # SQLite DB CRUD
│   ├── commands/            # Slash commands
│   │   ├── general.rs       # ping, info, stats
│   │   ├── fun.rs           # 24 fun commands
│   │   └── misc.rs          # 8 misc commands
│   ├── events/mod.rs        # Event handlers
│   ├── modules/             # Feature modules
│   │   ├── auto_mod.rs      # Auto-moderation
│   │   ├── logging.rs       # Message/voice/join logging
│   │   ├── moderation/      # Ban/kick/warn/mute/purge etc
│   │   ├── custom_commands.rs # Lua scripting
│   │   ├── giveaway.rs      # Giveaways
│   │   ├── tickets.rs       # Ticket system
│   │   ├── xp.rs            # Leveling / XP
│   │   ├── afk.rs           # AFK status
│   │   ├── ranks.rs         # Self-assignable roles
│   │   ├── reminders.rs     # Reminder polling
│   │   ├── scheduling.rs    # Temp actions & announcements
│   │   └── manager.rs       # Mod role management
│   └── web/mod.rs           # Axum web dashboard
├── config.example.toml      # Example configuration
└── Cargo.toml               # Dependencies
```

## Configuration

```toml
[discord]
token = "..."              # Bot token
client_id = "..."           # OAuth2 client ID
client_secret = "..."       # OAuth2 client secret
owner_id = "..."            # Discord user ID (optional)

[web]
port = 3000                 # Dashboard port (default: 3000)

[database]
url = "sqlite:retina.db"    # SQLite path (default)
```

## Dependencies

| Crate | Purpose |
|---|---|
| poise 0.6 | Slash command framework |
| serenity 0.12 | Discord API client |
| axum 0.7 | Web framework |
| tokio 1 | Async runtime |
| sqlx 0.7 | SQLite with migrations |
| chrono 0.4 | Timestamps |
| clap 4 | CLI arguments |
| toml 0.8 | Config deserialization |
| oauth2 5.0 | Discord OAuth2 login |
| reqwest 0.12 | HTTP client |
| mlua | Lua scripting |
| grass 0.13 | SCSS compiler (build) |
