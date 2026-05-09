# Discord Bot with Web Dashboard

A modular Discord bot written in Rust with a web dashboard.

(NOTE: The bot is mostly written by hand, 
   OpenCode mainly does the management and overview, 
   as well as tracking the todo's, 
   some code is written by it,
   but the code written by it is always double checked)

## Features

- **Modular Architecture**: Separate modules for bot logic and web dashboard
- **Slash Commands**: `/ping`, `/info`, `/stats`
- **Web Dashboard**: View bot statistics in real-time at `http://localhost:3000`
- **Shared State**: Bot and web dashboard share state via `Arc<RwLock<T>>`

## Setup

1. **Create a Discord Bot**:
   - Go to https://discord.com/developers/applications
   - Create a new application
   - Go to "Bot" section and create a bot
   - Enable "MESSAGE CONTENT INTENT" under Privileged Gateway Intents
   - Copy the bot token

2. **Invite Bot to Server**:
   - Go to "OAuth2" > "URL Generator"
   - Select "bot" scope
   - Select permissions: "Send Messages", "Embed Links"
   - Use the generated URL to invite the bot

3. **Configure Environment**:
   ```bash
   cp .env.example .env
   # Edit .env and add your DISCORD_TOKEN
   ```

4. **Run the Bot**:
   ```bash
   cargo run
   ```

## Usage

### Bot Commands
- `/ping` - Check bot latency
- `/info` - Display bot information
- `/stats` - View bot statistics

### Web Dashboard
Visit `http://localhost:3000` to see:
- Commands executed
- Dashboard visits
- Bot status

API endpoint: `http://localhost:3000/api/stats`

## Project Structure

```
discord-bot/
├── src/
│   ├── main.rs      # Entry point, runs bot and web server
│   ├── state.rs     # Shared application state
│   ├── bot/
│   │   └── mod.rs   # Discord bot logic and commands
│   └── web/
│       └── mod.rs   # Web dashboard with Axum
├── .env             # Environment variables
└── Cargo.toml       # Dependencies
```

## Dependencies

- **poise** - Discord bot framework
- **serenity** - Discord API wrapper
- **axum** - Web framework
- **tokio** - Async runtime
