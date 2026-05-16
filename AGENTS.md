# Retina — Project Context

## Vision

**Retina** is a full-featured Discord bot written in Rust, aiming to be a feature-complete clone of **Dyno** (https://www.dyno.gg). The bot and web dashboard run in the same process, sharing state via `Arc<RwLock<T>>`.

Tech stack: **Rust**, **poise** (Discord slash commands), **serenity** (Discord API), **axum** (web server), **sqlx + SQLite** (database), **tokio** (async runtime).

**Current status:** All major Dyno feature categories implemented. **99** slash commands total. Web dashboard for configuration. No outstanding build errors.

---

## Goal
- Continue developing the Retina Discord bot: add remaining Dyno parity commands across all categories, build the web dashboard out further, harden existing code.

## Constraints & Preferences
- Rust, poise, serenity, axum, sqlx+SQLite, tokio stack
- Web dashboard for configuration
- All modules toggleable per-guild
- Custom commands are Lua-only

## Progress

### Done (earlier sessions)
- Removed `boa_engine` and `pyo3` from Cargo.toml; dropped `python` feature
- Stripped `execute_js` and `execute_python`/`execute_python_impl` from `custom_commands.rs`; run() now always calls `execute_lua`
- Removed `language` field from `CustomCommand` struct, all SQL queries, `set_custom_command()` signature
- Deleted migration `20250116000000_add_custom_command_language.sql`
- Removed language dropdown and language badge logic from web UI and `dashboard.js`
- Updated wiki/server config descriptions to only mention Lua
- Created migration `20250117000000_extend_commands.sql` with tables: `moderator_roles`, `afk_status`, `reminders`, `self_roles`, `mod_notes`
- Added all DB CRUD methods for new tables to `database/mod.rs` (+ `edit_warning`, `get_warning_by_id`)
- Created `src/commands/fun.rs`: 24 slash commands — `rps`, `flip`, `roll`, `dadjoke`, `cat`, `dog`, `pug`, `github`, `urban`, `_8ball`, `meme`, `number`, `roast`, `yomama`, `norris`, `pokemon`, `wouldyourather`, `space`, `translate`, `weather`, `remindme`, `timer`, `choose`, `poll`
- Created `src/commands/misc.rs`: 8 slash commands — `avatar`, `whois`, `serverinfo`, `membercount`, `randomcolor`, `invite`, `prefix`, `emotes`
- Created `src/modules/afk.rs`: `afk`, `afk_list` commands + `check_afk_mention` and `check_afk_return` event handlers
- Created `src/modules/ranks.rs`: `addrank`, `delrank`, `rank_cmd` (join/leave toggle), `ranks` commands
- Created `src/modules/reminders.rs`: background `run_reminder_checker` task (fires every 30s)
- Created `src/modules/moderation/extended.rs`: 11 commands — `softban`, `members`, `move`, `voicekick`, `deafen`, `vmute`, `reason`, `case`, `notes` (add/list/delete/edit), `clearwarn`, `delwarn`
- Created `src/modules/manager.rs`: 6 commands — `addmod`, `delmod`, `listmods`, `nick`, `addrole`, `delrole`
- Wired all new modules into `commands/mod.rs`, `events/mod.rs`, `modules/mod.rs`, `main.rs`
- Build compiles with zero errors

### Done (this session)
- Added `admin_ids: Vec<u64>` to `Config` and `admin_ids: Vec<String>` to `DiscordSettings` in `config/mod.rs`
- Added `admin_ids` parsing in `main.rs` from config TOML
- Added `GuildInfo` struct (name, owner_id, icon) and `guild_info: HashMap<String, GuildInfo>` to `BotState` in `types.rs`
- Populated `guild_info` in event handlers (`Ready`, `GuildCreate`, `GuildDelete`) in `events/mod.rs`
- Added `/admin` route + `admin_handler` in `web/mod.rs` — bot admin panel with live stats and guild list with icons/names/owners
- Added `is_user_admin()` helper to check `config.admin_ids`
- Added `{{ADMIN_NAV_LINK}}` to nav partials for admin users
- Created `templates/partials/admin_content.html`
- Fixed `server_dashboard_handler` to check guild permissions (MANAGE_GUILD/ADMINISTRATOR) — returns 403 for members without admin privs, bot admins bypass
- Updated README comprehensively
- Added 5 truth-or-dare commands (`/truth`, `/dare`, `/wyr`, `/nhie`, `/paranoia`) using `api.truthordarebot.xyz` in `src/commands/fun.rs`
- Made `/wouldyourather` an alias for `/wyr` (both call shared `wyr_impl` helper with optional `rating` param)
- Updated `commands_content.html` with the 5 new command entries in the Fun section
- Updated `wiki_content.html` Fun command count from 24+ → 29+

### In Progress / Next
- (nothing currently in progress)

---

## Dyno Feature Map — Target State

| Feature | Status | Notes |
|---|---|---|
| **Moderation** (ban/kick/mute/warn/purge/slowmode/lockdown) | ★ Complete | + softban, members, move, voicekick, deafen, vmute, reason, case, notes, clearwarn, delwarn |
| **Auto-mod** (spam, caps, links, emotes, mentions, banned words) | ★ Complete | all 7 rule types with 5 action types (delete/warn/timeout/kick/ban) |
| **Custom Commands** | ★ Complete | Lua-scripted custom commands via !prefix with Discord API bindings |
| **Reaction Roles** | ★ Complete | emoji-to-role mapping on messages, add/remove on reaction |
| **Logging** (msg edits/deletes, join/leave, voice, mod actions) | ★ Complete | message edits/deletes, member join/leave, member update, channel create/delete, voice state |
| **Welcome / Goodbye** | ★ Complete | customizable join/leave messages with {user}, {mention}, {guild} templates |
| **Giveaways** | ★ Complete | embed with reaction entry, random winner picker, scheduled draw, startup expiry check, web dashboard list |
| **Tickets** | ★ Complete | panel with 🎫 reaction, private channel creation, close/claim/add/remove, staff role, web dashboard toggle |
| **Leveling / XP** | ★ Complete | message XP with cooldown, level-up role rewards, rank/leaderboard/set/add commands, web dashboard toggle |
| **Fun / Misc** | ★ Complete | 29 fun commands (rps, flip, roll, dadjoke, cat, dog, pug, github, urban, 8ball, meme, number, roast, yomama, norris, pokemon, wouldyourather, space, translate, weather, remindme, timer, choose, poll, truth, dare, wyr, nhie, paranoia) + 8 misc commands (avatar, whois, serverinfo, membercount, randomcolor, invite, prefix, emotes) |
| **AFK** | ★ Complete | afk/afk_list commands, mention detection, auto-remove on message |
| **Self-Assignable Roles** | ★ Complete | addrank/delrank/rank/ranks commands, toggle join/leave |
| **Reminders** | ★ Complete | background 30s poll loop, remindme command |
| **Manager** | ★ Complete | addmod/delmod/listmods, nick, addrole/delrole |
| **Scheduling** | ★ Complete | tempban, tempmute, scheduled announcements with interval repeats |
| **Web Dashboard** (per-server + admin panel) | ★ Complete | OAuth login with session cookie, guild picker, module toggles, auto-mod config, welcome, reaction-roles, custom-commands, giveaways, tickets, XP/leveling config, bot-in-server detection, invite flow, commands & wiki pages, POST endpoints (no auth yet) + `/admin` bot admin panel with live stats and guild list + permission enforcement (403 for non-admin members on server config) |

---

## Project Structure

```
discord-bot/
├── build.rs                 # Compiles static/style.scss → CSS at build time (grass)
├── migrations/              # SQLite migrations (sqlx)
│   ├── 20250101000000_initial.sql
│   ├── 20250102000000_create_sessions.sql
│   ├── 20250103000000_add_welcome.sql
│   ├── 20250104000000_create_custom_commands.sql
│   ├── 20250105000000_add_custom_commands_module.sql
│   ├── 20250106000000_create_reaction_roles.sql
│   ├── 20250107000000_add_reaction_roles_module.sql
│   ├── 20250108000000_create_giveaways.sql
│   ├── 20250109000000_create_ticket_tables.sql
│   ├── 20250110000000_add_tickets_module.sql
│   ├── 20250111000000_create_xp_tables.sql
│   ├── 20250112000000_add_xp_module.sql
│   ├── 20250113000000_create_scheduled_announcements.sql
│   ├── 20250114000000_create_scheduled_actions.sql
│   ├── 20250115000000_add_scheduling_module.sql
│   └── 20250117000000_extend_commands.sql
├── static/
│   ├── style.scss           # All dashboard styles (SCSS with variables & nesting)
│   └── dashboard.js         # Client-side JS (toggleModule, saveAutoMod, rule UI)
├── templates/
│   ├── index.html           # Landing page shell ({{STYLE}} {{TITLE}} {{CONTENT}})
│   ├── dashboard.html       # Server list shell
│   ├── server.html          # Per-server config shell (+ {{SCRIPT}})
│   └── partials/            # Reusable HTML fragments for all three pages
│       ├── index_logged_in.html
│       ├── index_anonymous.html
│       ├── server_card.html
│       ├── server_no_servers.html
│       ├── dashboard_content.html
│       ├── server_config.html
│       ├── rule_card.html
│       ├── commands_content.html
│       ├── dashboard_content.html
│       ├── index_anonymous.html
│       ├── index_logged_in.html
│       ├── rule_banned_words_fields.html
│       ├── rule_caps_fields.html
│       ├── rule_card.html
│       ├── rule_emotes_fields.html
│       ├── rule_max_length_fields.html
│       ├── rule_mentions_fields.html
│       ├── rule_spam_fields.html
│       ├── server_card.html
│       ├── server_config.html
│       ├── server_no_servers.html
│       ├── wiki_content.html
│       └── admin_content.html
├── src/
│   ├── main.rs              # Entrypoint — parses CLI, loads config.toml, starts bot & web + spawns scheduler & reminder checker
│   ├── cli.rs               # Clap CLI definition (--config path)
│   ├── types.rs             # AppState with bot_state, web_state, config, settings, db, spam_tracker
│   ├── config/mod.rs        # Config, GuildConfig, AutoModConfig, WelcomeConfig + Settings (from config.toml)
│   ├── database/mod.rs      # Database — guild configs, warnings, sessions, giveaways, xp, reaction_roles, custom_commands, tickets, + 5 new tables (moderator_roles, afk_status, reminders, self_roles, mod_notes) + scheduling
│   ├── commands/
│   │   ├── mod.rs           # Aggregates all slash commands (general, fun, misc, moderation, giveaway, tickets, xp, scheduling, afk, ranks, manager)
│   │   ├── general.rs       # /ping, /info, /stats
│   │   ├── fun.rs           # 29 fun commands (rps, flip, roll, dadjoke, cat, dog, pug, github, urban, 8ball, meme, number, roast, yomama, norris, pokemon, wouldyourather, space, translate, weather, remindme, timer, choose, poll, truth, dare, wyr, nhie, paranoia)
│   │   └── misc.rs          # 8 misc commands (avatar, whois, serverinfo, membercount, randomcolor, invite, prefix, emotes)
│   ├── events/mod.rs        # Event handler — Ready, Message, GuildCreate, GuildDelete, MessageDelete, MessageUpdate, GuildMemberAddition/Removal/Update, ChannelCreate/Delete, ReactionAdd/Remove, VoiceStateUpdate (+ AFK checks)
│   ├── modules/
│   │   ├── mod.rs           # init_modules() — logs which modules are enabled
│   │   ├── afk.rs           # afk/afk_list commands, check_afk_mention/check_afk_return event handlers
│   │   ├── auto_mod.rs      # All 7 rule types with 5 action types (delete/warn/timeout/kick/ban)
│   │   ├── custom_commands.rs # Lua-scripted !commands via mlua with Discord API bindings
│   │   ├── giveaway.rs      # /giveaway create/reroll/end, reaction entry, random picker, timers
│   │   ├── logging.rs       # Full logging: msg edits/deletes, join/leave, channel, voice
│   │   ├── manager.rs       # addmod/delmod/listmods, nick, addrole/delrole
│   │   ├── moderation/
│   │   │   ├── mod.rs       # /ban/kick/warn/warnings/mute/purge/slowmode/lockdown
│   │   │   └── extended.rs  # softban, members, move, voicekick, deafen, vmute, reason, case, notes, clearwarn, delwarn
│   │   ├── ranks.rs         # addrank/delrank/rank/ranks self-assignable roles
│   │   ├── reminders.rs     # run_reminder_checker background task (30s poll)
│   │   ├── scheduling.rs    # tempban, tempmute, scheduled announcements with interval repeats
│   │   ├── tickets.rs       # /ticket setup/close/claim/add/remove, panel reaction create, permissions
│   │   └── xp.rs            # /rank /leaderboard /xp set/add/role, message XP handler, level-up roles
│   └── web/mod.rs           # Axum dashboard — 18 routes, render_page(), template partials
├── config.example.toml      # Example config file (discord, web, database sections)
├── cert.pem / key.pem       # SSL certs (present but NOT used — redirect URL is http)
└── Cargo.toml
```

---

## Key Components

### 1. `cli.rs` — CLI Parser
- Uses `clap` derive macro
- `--config`, `-c` — path to config file (default: `config.toml`)
- Easily extensible with more flags as needed

### 2. `main.rs` — Entrypoint
- Parses CLI args via `Cli::parse()`
- Loads `Settings` from TOML config file (exits with error + hint if not found)
- Initializes Database (sqlx + SQLite, runs migrations including sessions and extend_commands tables)
- Populates `Config.owner_ids` from `settings.discord.owner_id` and `Config.admin_ids` from `settings.discord.admin_ids`
- Creates `AppState` with shared `BotState`, `WebState`, `Config`, `Settings`, `db`, and `spam_tracker`
- Calls `modules::init_modules()` to print startup status
- Spawns bot in a separate task, then blocks on web server
- Spawns giveaway expiry checker (10s delay), scheduler (15s delay), and reminder checker (5s delay)
- **If web server stops, the whole process dies** (the bot task is detached)

### 3. `types.rs` — Shared State
- `AppState` — cloned everywhere, contains:
  - `bot_state` — `commands_executed` counter, `started_at` timestamp, `bot_guilds: HashSet<String>`, `guild_info: HashMap<String, GuildInfo>` (name, owner_id, icon — populated via GuildCreate/Ready events)
  - `web_state` — `visits` counter
  - `config` — runtime bot config (prefix, owner_ids, module toggles; in-memory only)
  - `settings` — loaded from `config.toml` (read-only after startup; discord token, OAuth2 creds, web port, DB url)
  - `db` — `Database` struct wrapping `SqlitePool`
  - `spam_tracker` — `HashMap<String, Vec<Instant>>` for spam detection
- Session store for web OAuth is **not** in AppState; it's created locally in `web::start()` and loaded from DB on boot
- All guild configs are fetched on-demand from SQLite via `state.db.*`

### 4. `config/mod.rs` — Configuration structs
- **`Settings`** (from `config.toml`):
  - `discord`: `token`, `client_id`, `client_secret`, `owner_id` (optional), `admin_ids` (optional vector of user ID strings)
  - `web`: `host` (default: `0.0.0.0`), `port` (default: 3000)
  - `database`: `url` (default: `sqlite:retina.db`)
  - Loaded via `Settings::load("config.toml")` — reads file + deserializes with `toml` + `serde`
- **`Config`** — runtime bot config: `prefix`, `owner_ids`, `admin_ids`, `ModulesConfig`. In-memory defaults (no persistence).
- **`GuildConfig`** — per-guild: `guild_id`, `ModulesConfig`, `AutoModConfig`, `WelcomeConfig`. Persisted in SQLite via `Database`.
- **`AutoModConfig`** — manual `Default`: `enabled: false`, `max_message_length: 2000`, `banned_words: []`, `ban_duration_hours: 24`
- **`WelcomeConfig`** — per-guild welcome/goodbye: `enabled`, `welcome_channel_id`, `goodbye_channel_id`, `welcome_message`, `goodbye_message`. Stored as JSON in `welcome_config` column.

### 5. `commands/` — Slash Commands
- **general.rs**: `/ping` (latency), `/info` (commands count + guild count), `/stats` (uptime + commands)
- **fun.rs**: 29 fun commands using external APIs and static lists
- **misc.rs**: 8 utility/info commands
- Commands are registered globally via `poise::builtins::register_globally()` (slow propagation, ~1hr)

### 6. `events/mod.rs` — Event Handling
- `Ready` → sets `started_at` timestamp, populates `bot_guilds` + `guild_info` from serenity cache
- `GuildCreate` → adds guild ID + guild info (name, owner_id, icon) to `bot_state`
- `GuildDelete` → removes guild ID + guild info from `bot_state`
- `Message` → checks per-guild auto-mod rules, runs moderation, custom commands, XP handler, AFK return check, AFK mention detection
- `MessageDelete` → logs to `#mod-logs` if logging module enabled
- `MessageUpdate` → logs edits if logging enabled
- `GuildMemberAddition` → sends welcome message + logs join
- `GuildMemberRemoval` → sends goodbye message + logs leave
- `GuildMemberUpdate` → logs role changes
- `ChannelCreate` / `ChannelDelete` → logs channel changes
- `ReactionAdd` → role assignment, giveaway entry, ticket creation
- `ReactionRemove` → role removal, giveaway removal
- `VoiceStateUpdate` → logs voice events

### 7. `modules/` — Feature Modules
- **auto_mod.rs**: AutoMod struct with `run()` checking all 7 rule types. Dispatches 5 action types. Spam detection uses in-memory `spam_tracker`.
- **logging.rs**: Full logging module. All logs sent to `#mod-logs` channel as embeds.
- **moderation/mod.rs**: Core moderation commands. `handle_message()` is a no-op placeholder.
- **moderation/extended.rs**: Extended moderation commands (softban, members, move, voicekick, deafen, vmute, reason, case, notes, clearwarn, delwarn).
- **afk.rs**: AFK tracking with mention detection and auto-remove on user message.
- **ranks.rs**: Self-assignable roles via toggle command.
- **reminders.rs**: Background poll loop checking for due reminders every 30s.
- **manager.rs**: Moderator role management, nickname changes, role assignment.
- **scheduling.rs**: tempban/tempmute via scheduled_actions table + scheduled announcements with interval repeats.
- **giveaway.rs**: Full giveaway lifecycle with reaction entry, random winner, scheduled end.
- **tickets.rs**: Ticket system with panel, private channels, claim/close/add/remove.
- **xp.rs**: Message XP with cooldown, level-up roles, leaderboards.

### 8. `web/mod.rs` — Web Dashboard (~1300 lines)
- Routes: `/`, `/commands`, `/wiki`, `/login`, `/auth/callback`, `/logout`, `/dashboard`, `/server/:guild_id`, `/server/:guild_id/toggle`, `/server/:guild_id/automod`, `/server/:guild_id/welcome`, `/server/:guild_id/custom_command`, `/server/:guild_id/reaction_role`, `/server/:guild_id/xp_config`, `/server/:guild_id/ticket`, `/server/:guild_id/xp_reward`, `/admin`, `/api/stats`, `/api/modules`
- Template system with compile-time includes
- OAuth2 flow with DB-persisted sessions (24h expiry)
- Bot-in-server detection for invite/manage UI
- Dashboard shows guilds where user has `MANAGE_SERVER` or `ADMINISTRATOR` permission
- Server config page checks permissions on every access (returns 403 for non-admin members)
- Bot admins (in `admin_ids`) bypass per-guild permission checks
- `/admin` route for bot admin panel with live global stats and guild list

### 9. `database/mod.rs` — SQLite Database (sqlx)
- Connection pooling (max 5), migrations auto-run on init
- **Guild Config CRUD** — get/set with JSON auto_mod and welcome config
- **Warnings CRUD** — add, get, delete, clear, count, edit, get_by_id
- **OAuth Sessions** — store, load valid, remove
- **Reaction Roles** — list, add, remove
- **Custom Commands** — get, list, set, delete (Lua only)
- **Giveaways CRUD** — create, get, list, update entries, end
- **Tickets** — config CRUD, ticket CRUD, status update
- **XP** — config CRUD, data upsert, leaderboard, rewards
- **Scheduling** — scheduled announcements CRUD, scheduled actions CRUD, due queries
- **Moderator Roles** — add, remove, list, check
- **AFK** — set, remove, get, list
- **Reminders** — create, get due, delete, list
- **Self Roles** — add, remove, list, get by name
- **Mod Notes** — add, list, delete, edit

---

## Data Flow

1. CLI parses args → loads `config.toml` → inits DB → runs migrations → spawns bot + scheduler + reminder checker → starts web server
2. All guild config data persists across restarts in `retina.db`
3. OAuth sessions survive restarts (24h window)
4. Bot receives message → fetches per-guild config from DB → runs auto-mod → moderation → custom commands → XP → AFK checks

---

## Known Issues & Technical Debt

### Critical
- **No auth on POST endpoints** — `toggle_handler` and `automod_handler` don't verify user session
- **Token in URL query param** — `?t=<token>` is insecure (tokens in URLs, logs, browser history)

### Moderate
- **Global config not persisted** — `Config` struct uses code defaults; no way to change prefix/owner_ids/module defaults without recompiling
- **Per-guild configs lazy-created** — inserted on first web dashboard visit or first event; no pre-seeding
- **Bot dies if web server crashes** — `start_bot` is spawned as a detached task; if web server returns, `main()` exits
- **Global command registration** — uses `register_globally` (slow); should be per-guild for development
- **`handle_message` in moderation module** — checks if module is enabled but does nothing else; placeholder
- **No error handling in event handler** — `handle_event` returns `Ok(())` on all paths, errors are swallowed
- **No HTTPS** — SSL certs exist but redirect URL in code is hardcoded `http://localhost:3000/auth/callback`
- **sqlx `migrate!` is compile-time** — migrations directory must exist at build time; adding migrations requires recompilation

---

## Development Roadmap (suggested order)

1. ~~**Replace database** — SQLite (via `sqlx` or `rusqlite`) for persistence~~ ✅ Done
2. ~~**Complete auto-mod** — add spam/caps/links/emotes/mentions filters + configurable actions (delete, warn, mute, kick, ban)~~ ✅ Done
3. ~~**Complete moderation** — mute (timeout), purge, slowmode, lockdown~~ ✅ Done
4. ~~**Complete logging** — message edits, voice events, member join/leave, role changes, channel changes~~ ✅ Done
5. ~~**Welcome / Goodbye** — customizable join/leave messages~~ ✅ Done
6. ~~**Custom Commands** — per-guild Lua-scripted commands~~ ✅ Done
7. ~~**Reaction Roles** — assign roles on reaction~~ ✅ Done
8. ~~**Giveaways** — embed with reaction entry, random picker, scheduled draw~~ ✅ Done
9. ~~**Tickets** — panel with 🎫 reaction, private channel, close/claim/add/remove~~ ✅ Done
10. ~~**Leveling / XP** — message XP with cooldown, leaderboards, role rewards~~ ✅ Done
11. ~~**Scheduling** — timed announcements, tempban, tempmute~~ ✅ Done
12. ~~**Web dashboard expansion** — full per-server config UI for every module, auth on all POST endpoints, session cookie~~ ✅ Done
13. ~~**Fun / Misc Commands** — 24 fun + 8 misc commands integrating external APIs~~ ✅ Done
14. ~~**AFK** — afk/afk_list with mention detection and auto-remove~~ ✅ Done
15. ~~**Self-Assignable Roles** — addrank/delrank/rank/ranks~~ ✅ Done
16. ~~**Reminders** — background checker, remindme command~~ ✅ Done
17. ~~**Extended Moderation** — softban, members, move, voicekick, deafen, vmute, reason, case, notes, clearwarn, delwarn~~ ✅ Done
18. ~~**Manager Commands** — addmod/delmod/listmods, nick, addrole/delrole~~ ✅ Done

---

## Conventions & Patterns

- **State sharing**: Everything goes through `AppState` with `Arc<RwLock<T>>` (tokio RwLock)
- **Module pattern**: Each module has a struct with `new()` and methods; currently stateless, will need DB access
- **Commands**: Defined with `#[poise::command(slash_command)]` macro, permission-gated with `required_permissions`
- **Web routes**: Axum with `State((app_state, session_store))` tuple extraction
- **Error type**: `type Error = Box<dyn std::error::Error + Send + Sync>`
- **CLI**: `clap` derive in `src/cli.rs`. `Cli::parse()` in main. Extend by adding fields to the `Cli` struct.
- **App-level config**: `Settings` from `config.toml` — loaded once at startup, stored in `AppState.settings`. Read-only.
- **Bot runtime config**: `Config` struct — in-memory only (prefix, owner_ids, modules). Not persisted.
- **Guild configs**: `serde` data structs persisted via `sqlx` + SQLite. Stored in `guild_configs` table. Module toggles as INTEGER columns, `AutoModConfig` as JSON in `auto_mod_config` TEXT column.
- **DB access**: Via `state.db` field on `AppState`. Database is `Clone` (pool is ref-counted internally). Migrations in `migrations/` run at startup.
- **HTML templating**: Page shells in `templates/*.html`, content fragments in `templates/partials/*.html`. Loaded via `include_str!()` at compile time. Placeholder syntax: `{{PLACEHOLDER}}`.
- **CSS/SCSS**: Write SCSS in `static/style.scss`. Compiled to CSS at build time via `build.rs` + `grass`. CSS is inlined as `<style>` tag in HTML (no external CSS request).
- **JS**: Client-side JS in `static/dashboard.js`. Served at `/static/dashboard.js` via `ServeDir`.
- **Build script**: `build.rs` watches `static/style.scss` changes, compiles SCSS to CSS into `$OUT_DIR/style.css`. Only re-runs when SCSS file changes.
- **Sessions**: OAuth sessions stored in `sessions` SQLite table. Loaded into memory on boot. 24-hour expiry checked on every request via `expires_at` field.
- **External APIs**: icanhazdadjoke.com, thecatapi.com, dog.ceo, pokeapi.co, api.github.com, numbersapi.com, open-notify.org, wttr.in, mymemory.translated.net, chucknorris.io, meme-api.com, urbandictionary.com, api.truthordarebot.xyz

---

## Configuration (`config.toml`)

All settings live in a TOML config file. Pass `--config <path>` to use a non-default location.

```toml
[discord]
token = "..."              # Required: Discord bot token
client_id = "..."           # Required: OAuth2 client ID
client_secret = "..."       # Required: OAuth2 client secret
owner_id = "..."            # Optional: Discord user ID for admin
admin_ids = ["..."]         # Optional: Discord user IDs for bot admin panel access

[web]
host = "0.0.0.0"            # Optional: web dashboard host (default: 0.0.0.0)
port = 3000                 # Optional: web dashboard port (default: 3000)

[database]
url = "sqlite:retina.db"    # Optional: SQLite connection string
```

---

## Build & Run

```bash
cp config.example.toml config.toml   # then edit with real values
cargo run                            # starts bot + web server (reads config.toml by default)
cargo run -- --config myconfig.toml  # use an alternative config file
```

Requires Rust edition 2021. Dependencies are in `Cargo.toml`.

---

## Main Dependencies

| Crate | Purpose |
|---|---|
| `poise 0.6` | Slash command framework |
| `serenity 0.12` | Discord API client |
| `axum 0.7` | Web framework |
| `tokio 1` | Async runtime |
| `sqlx 0.7` | SQLite database with migrations and connection pooling |
| `chrono 0.4` | Timestamp parsing, session expiry |
| `clap 4` | CLI argument parsing (--config) |
| `toml 0.8` | Config file deserialization |
| `oauth2 5.0` | Discord OAuth2 login flow |
| `reqwest 0.12` | HTTP client (OAuth token exchange, Discord API calls, fun commands) |
| `tracing` | Structured logging |
| `serde` / `serde_json` | Serialization |
| `tower-http` | Static file serving (features = ["full"]) |
| `rustls` / `tokio-rustls` | TLS (present but unused) |
| `rustls-pemfile 1.0` | PEM file parsing for TLS |
| `tower 0.4` | Service middleware layer |
| `tracing-subscriber 0.3` | Logging subscriber |
| `base64 0.22` | Base64 encoding for session tokens |
| `mlua 0.11` | Lua scripting for custom commands (lua54, send) |
| `rand 0.8` | Random number generation (giveaways, fun commands) |
| `grass 0.13` | SCSS-to-CSS compiler (build dependency via build.rs) |
