# Retina — Discord Bot

A full-featured Discord bot written in Rust, aiming to be a feature-complete clone of **Dyno**. The bot and web dashboard run in the same process, sharing state via `Arc<RwLock<T>>`.

**Tech stack:** Rust edition 2021, poise 0.6 (slash commands), serenity 0.12 (Discord API), axum 0.7 (web server), sqlx 0.7 + SQLite (database), tokio 1 (async runtime), mlua 0.11 (Lua scripting).

## Prerequisites

- **Rust** 1.75+ (edition 2021)
- **SQLite development headers** (`libsqlite3-dev` on Debian/Ubuntu, `sqlite-devel` on Fedora)
- **OpenSSL development headers** (`libssl-dev` on Debian/Ubuntu, `openssl-devel` on Fedora)

## Features

### Moderation — 19 commands
`/ban` `/kick` `/warn` `/warnings` `/mute` `/purge` `/slowmode` `/lockdown` `/softban` `/members` `/move` `/voicekick` `/deafen` `/vmute` `/reason` `/case` `/notes` `/clearwarn` `/delwarn`

### Auto-moderation
7 rule types: spam, excessive caps, link detection, mass mentions, emote spam, banned words, max message length. 5 action types: delete, warn, timeout, kick, ban. Channel and role whitelist/blacklist per guild.

### Custom Commands
Lua-scripted commands via `!<name>` prefix. API bindings: `api.send()`, `api.reply()`, `api.ban()`, `api.kick()`, `api.timeout()`, `api.warn()`, `api.add_role()`, `api.remove_role()`, `api.has_role()`, `api.embed()`, `api.react()`, `api.delete()`, `api.edit()`. Access `message.author_id`, `message.content`, `args[1..]`, and more.

### Reaction Roles
Emoji-to-role mapping on messages. Automatic role add/remove on reaction add/remove.

### Logging
Message edits and deletions, member join/leave/update, channel create/delete, voice state updates. All sent to `#mod-logs` as embeds.

### Welcome / Goodbye
Customizable join and leave messages with `{user}`, `{mention}`, `{guild}` template variables. Per-guild channel selection.

### Giveaways
Embed with reaction entry, random winner picker, scheduled end timers, startup expiry check, web dashboard list.

### Tickets
Reaction-based panel (`🎫`), private channel creation, close/claim/reopen/add/remove, staff role config, web dashboard management.

### Leveling / XP
Per-message XP with cooldown, minimum character threshold, level-up role rewards, `/rank` and `/leaderboard` commands, web dashboard config.

### Scheduling
Tempban and tempmute via timed actions table. Scheduled announcements with interval repeats (e.g. every 24h).

### General — 3 commands
`/ping` (latency), `/info` (commands + guild count), `/stats` (uptime + counters)

### Fun — 24 commands
`/rps` `/flip` `/roll` `/dadjoke` `/cat` `/dog` `/pug` `/github` `/urban` `/8ball` `/meme` `/number` `/roast` `/yomama` `/norris` `/pokemon` `/wouldyourather` `/space` `/translate` `/weather` `/remindme` `/timer` `/choose` `/poll`

External APIs: icanhazdadjoke, thecatapi, dog.ceo, pokeapi.co, api.github.com, numbersapi.com, open-notify.org, wttr.in, mymemory.translated.net, chucknorris.io, meme-api.com, urbandictionary.com

### Misc — 8 commands
`/avatar` `/whois` `/serverinfo` `/membercount` `/randomcolor` `/invite` `/prefix` `/emotes`

### AFK
`/afk` `/afk_list`. Auto-detects mentioned AFK users, auto-removes AFK status on user message.

### Self-Assignable Roles
`/addrank` `/delrank` `/rank` (join/leave toggle) `/ranks`. Web dashboard config.

### Reminders
Background task polling every 30s for due reminders. `/remindme` command.

### Manager
`/addmod` `/delmod` `/listmods` `/nick` `/addrole` `/delrole`

### Web Dashboard
OAuth2 login with Discord, guild picker filtered to servers where you have MANAGE_GUILD or ADMINISTRATOR, per-server module toggles (9 modules), auto-mod rule editor with all 7 rule types, welcome/goodbye message editor, custom command editor with Lua script editing, reaction role manager, giveaway list, ticket management (close/claim/reopen), XP config with role rewards.

### Bot Admin Panel
Accessible at `/admin` for users in the `admin_ids` config list. Shows live global stats (warnings, custom commands, active giveaways, open tickets, reaction roles, XP users, configured guilds, uptime), guild list with icons/names/owner IDs, and quick-manage links to any server (bypasses guild permission checks).

## Setup

1. **Create a Discord Bot**:
   - Go to https://discord.com/developers/applications
   - Create a new application
   - Go to "Bot" section, create a bot, enable **MESSAGE CONTENT INTENT** and **SERVER MEMBERS INTENT**
   - Copy the bot token

2. **OAuth2 Setup**:
   - In "OAuth2" > "General", copy Client ID and Client Secret
   - Add redirect URL: `http://localhost:3000/auth/callback`

3. **Invite Bot**:
   - In "OAuth2" > "URL Generator", select `bot` and `applications.commands` scopes
   - Permissions: Administrator (recommended) or select specific permissions
   - Use the generated URL to invite the bot to your server

4. **Configure**:
   ```bash
   cp config.example.toml config.toml
   # Fill in your token, client_id, client_secret in config.toml
   ```

5. **Run**:
   ```bash
   cargo run
   # Or with a custom config path:
   cargo run -- --config myconfig.toml
   ```

## Configuration

```toml
[discord]
token = "..."                    # Bot token (required)
client_id = "..."                # OAuth2 client ID (required)
client_secret = "..."            # OAuth2 client secret (required)
owner_id = "..."                 # Discord user ID for bot ownership (optional)
admin_ids = ["111111111111"]     # Users who can access /admin panel (optional)

[web]
host = "0.0.0.0"                 # Dashboard bind address (default: 0.0.0.0)
port = 3000                      # Dashboard port (default: 3000)

[database]
url = "sqlite:retina.db"         # SQLite connection string (default: sqlite:retina.db)
```

Slash commands are registered **globally** on startup (may take up to 1 hour to propagate). All guild config is persisted in the SQLite database across restarts.

## Web Dashboard

Visit `http://localhost:3000` and log in with Discord.

### Auth Model
- OAuth2 login with `identify` and `guilds` scopes
- Sessions stored in SQLite, loaded on boot, cached in memory
- 24-hour session expiry, checked on every request
- Token passed via query param `?t=<token>` and `session` cookie
- Dashboard shows guilds where you have **MANAGE_GUILD** or **ADMINISTRATOR** permission
- Server config page checks permissions on every access (returns 403 if unauthorized)
- Bot admins (in `admin_ids`) bypass guild permission checks

### Routes

| Route | Method | Description |
|---|---|---|
| `/` | GET | Landing page with live stats |
| `/commands` | GET | Slash command reference |
| `/wiki` | GET | Wiki / documentation |
| `/login` | GET | Discord OAuth2 login |
| `/auth/callback` | GET | OAuth2 callback |
| `/logout` | GET | Clear session |
| `/dashboard` | GET | Guild picker (requires auth) |
| `/server/:id` | GET | Per-guild config (requires admin perms or bot admin) |
| `/server/:id/toggle` | POST | Toggle module |
| `/server/:id/automod` | POST | Save auto-mod rules |
| `/server/:id/welcome` | POST | Save welcome/goodbye config |
| `/server/:id/custom_command` | POST | Create/edit/delete custom command |
| `/server/:id/reaction_role` | POST | Add/delete reaction role |
| `/server/:id/xp_config` | POST | Save XP config |
| `/server/:id/ticket` | POST | Close/claim/reopen ticket |
| `/server/:id/xp_reward` | POST | Add/delete XP role reward |
| `/admin` | GET | Bot admin panel (requires admin_ids) |
| `/api/stats` | GET | Global bot stats JSON |
| `/api/modules` | GET | Global module defaults JSON |

**Note:** POST endpoints currently do **not** verify user session (no auth). This is a known issue.

## Project Structure

```
discord-bot/
├── build.rs                 # Compiles static/style.scss → CSS at build time (grass)
├── migrations/              # SQLite migrations (sqlx, compile-time via migrate!)
├── static/
│   ├── style.scss           # Dashboard SCSS styles
│   └── dashboard.js         # Client-side JS (toggleModule, saveAutoMod, etc.)
├── templates/
│   ├── index.html           # Landing page shell
│   ├── dashboard.html       # Server list shell
│   ├── server.html          # Per-server config shell
│   └── partials/            # Reusable HTML fragments (18 files)
├── src/
│   ├── main.rs              # Entrypoint — CLI, DB init, spawn bot + web + schedulers
│   ├── cli.rs               # Clap CLI definition (--config)
│   ├── types.rs             # AppState, BotState, GuildInfo, WebState
│   ├── config/mod.rs        # Config, Settings, GuildConfig, AutoModConfig, etc.
│   ├── database/mod.rs      # SQLite CRUD — guild configs, sessions, warnings,
│   │                        #   custom commands, reaction roles, giveaways,
│   │                        #   tickets, XP, scheduling, moderator_roles,
│   │                        #   afk_status, reminders, self_roles, mod_notes
│   ├── commands/
│   │   ├── mod.rs           # Aggregates all slash commands
│   │   ├── general.rs       # /ping /info /stats
│   │   ├── fun.rs           # 24 fun commands
│   │   └── misc.rs          # 8 misc commands
│   ├── events/mod.rs        # Event handler — Message, Ready, GuildCreate/Delete,
│   │                        #   MessageDelete/Update, GuildMember*, Channel*,
│   │                        #   Reaction*, VoiceState, AFK checks
│   ├── modules/
│   │   ├── mod.rs           # init_modules() — logs enabled modules
│   │   ├── afk.rs           # AFK tracking, mention detection, auto-remove
│   │   ├── auto_mod.rs      # All 7 rule types with 5 action types
│   │   ├── custom_commands.rs # Lua !commands via mlua
│   │   ├── giveaway.rs      # Giveaway lifecycle, reaction entry, random picker
│   │   ├── logging.rs       # Full event logging to #mod-logs
│   │   ├── manager.rs       # Moderator roles, nick, addrole/delrole
│   │   ├── moderation/
│   │   │   ├── mod.rs       # ban/kick/warn/mute/purge/slowmode/lockdown
│   │   │   └── extended.rs  # softban, members, move, voicekick, deafen, etc.
│   │   ├── ranks.rs         # Self-assignable roles
│   │   ├── reminders.rs     # Background 30s poll loop
│   │   ├── scheduling.rs    # Tempban, tempmute, scheduled announcements
│   │   ├── tickets.rs       # Ticket panel, private channels, claim/close
│   │   └── xp.rs            # Message XP, level-up roles, leaderboards
│   └── web/mod.rs           # Axum web dashboard — 18 routes, template rendering
├── config.example.toml      # Example configuration
└── Cargo.toml               # Dependencies
```

## Known Issues & Limitations

- **No auth on POST endpoints** — `toggle_handler`, `automod_handler`, etc. don't verify user session
- **Token in URL query param** — `?t=<token>` is visible in browser history and server logs
- **Global config not persisted** — `Config` (prefix, owner_ids, module defaults) uses code defaults, no way to change without recompiling
- **Bot dies if web server crashes** — bot runs in a detached task; if axum returns, `main()` exits
- **Global command registration** — uses `register_globally()` (up to 1hr propagation); per-guild guild would be better for development
- **No HTTPS** — SSL certs exist but redirect URL is hardcoded `http://localhost:3000/auth/callback`
- **sqlx compile-time migrations** — `migrate!` macro requires `migrations/` directory at build time; adding a migration requires recompilation

## Dependencies

| Crate | Purpose |
|---|---|
| poise 0.6 | Slash command framework |
| serenity 0.12 | Discord API client |
| axum 0.7 | Web framework |
| tokio 1 | Async runtime |
| sqlx 0.7 | SQLite with connection pooling and migrations |
| chrono 0.4 | Timestamps and session expiry |
| clap 4 | CLI argument parsing (--config) |
| toml 0.8 | Config file deserialization |
| oauth2 5.0 | Discord OAuth2 login flow |
| reqwest 0.12 | HTTP client (OAuth token exchange, Discord API, fun commands) |
| mlua 0.11 | Lua scripting for custom commands (lua54) |
| rand 0.8 | Random selection (giveaways, fun commands) |
| serde / serde_json | Serialization |
| tower-http 0.5 | Static file serving |
| base64 0.22 | Session token encoding |
| grass 0.13 | SCSS-to-CSS compiler (build dependency) |
| tracing / tracing-subscriber | Structured logging |
