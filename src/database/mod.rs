use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{FromRow, SqlitePool};
use crate::config::{AutoModConfig, GuildConfig, ModulesConfig};

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn init(database_url: &str) -> Result<Self, sqlx::Error> {
        let resolved = resolve_sqlite_url(database_url);
        println!("Connecting to database: {}", resolved);

        if let Some(path) = resolved.strip_prefix("sqlite:") {
            let p = std::path::Path::new(path);
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(p)
                .map_err(|e| {
                    eprintln!("Cannot create database file '{}': {}", p.display(), e);
                    sqlx::Error::Configuration(e.into())
                })?;
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&resolved)
            .await
            .map_err(|e| {
                eprintln!("Failed to connect to '{}': {}", resolved, e);
                e
            })?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Database { pool })
    }

    pub async fn get_guild_config(&self, guild_id: &str) -> Result<Option<GuildConfig>, sqlx::Error> {
        let row = sqlx::query_as::<_, GuildConfigRow>(
            "SELECT guild_id, moderation_enabled, auto_mod_enabled, logging_enabled, \
                    welcome_enabled, custom_commands_enabled, reaction_roles_enabled, \
                    tickets_enabled, xp_enabled, scheduling_enabled, \
                    auto_mod_config, welcome_config \
             FROM guild_configs WHERE guild_id = ?",
        )
        .bind(guild_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(GuildConfigRow::into_config))
    }

    pub async fn get_or_create_guild_config(&self, guild_id: &str) -> Result<GuildConfig, sqlx::Error> {
        if let Some(config) = self.get_guild_config(guild_id).await? {
            Ok(config)
        } else {
            let config = GuildConfig {
                guild_id: guild_id.to_string(),
                ..Default::default()
            };
            self.set_guild_config(&config).await?;
            Ok(config)
        }
    }

    pub async fn set_guild_config(&self, config: &GuildConfig) -> Result<(), sqlx::Error> {
        let auto_mod_json = serde_json::to_string(&config.auto_mod).unwrap_or_default();
        let welcome_json = serde_json::to_string(&config.welcome).unwrap_or_default();

        sqlx::query(
            "INSERT INTO guild_configs (guild_id, moderation_enabled, auto_mod_enabled, logging_enabled, \
                                        welcome_enabled, custom_commands_enabled, reaction_roles_enabled, \
                                        tickets_enabled, xp_enabled, scheduling_enabled, \
                                        auto_mod_config, welcome_config, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now')) \
             ON CONFLICT(guild_id) DO UPDATE SET \
                moderation_enabled = excluded.moderation_enabled, \
                auto_mod_enabled = excluded.auto_mod_enabled, \
                logging_enabled = excluded.logging_enabled, \
                welcome_enabled = excluded.welcome_enabled, \
                custom_commands_enabled = excluded.custom_commands_enabled, \
                reaction_roles_enabled = excluded.reaction_roles_enabled, \
                tickets_enabled = excluded.tickets_enabled, \
                xp_enabled = excluded.xp_enabled, \
                scheduling_enabled = excluded.scheduling_enabled, \
                auto_mod_config = excluded.auto_mod_config, \
                welcome_config = excluded.welcome_config, \
                updated_at = datetime('now')",
        )
        .bind(&config.guild_id)
        .bind(config.modules.moderation)
        .bind(config.modules.auto_mod)
        .bind(config.modules.logging)
        .bind(config.modules.welcome)
        .bind(config.modules.custom_commands)
        .bind(config.modules.reaction_roles)
        .bind(config.modules.tickets)
        .bind(config.modules.xp)
        .bind(config.modules.scheduling)
        .bind(&auto_mod_json)
        .bind(&welcome_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_warning(
        &self,
        guild_id: &str,
        user_id: &str,
        moderator_id: &str,
        reason: &str,
    ) -> Result<Warning, sqlx::Error> {
        sqlx::query(
            "INSERT INTO warnings (guild_id, user_id, moderator_id, reason) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(moderator_id)
        .bind(reason)
        .execute(&self.pool)
        .await?;

        let warning = sqlx::query_as::<_, Warning>(
            "SELECT id, guild_id, user_id, moderator_id, reason, created_at \
             FROM warnings WHERE rowid = last_insert_rowid()",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(warning)
    }

    pub async fn get_warnings(
        &self,
        guild_id: &str,
        user_id: &str,
    ) -> Result<Vec<Warning>, sqlx::Error> {
        sqlx::query_as::<_, Warning>(
            "SELECT id, guild_id, user_id, moderator_id, reason, created_at \
             FROM warnings WHERE guild_id = ?1 AND user_id = ?2 ORDER BY created_at DESC",
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn store_session(
        &self,
        token: &str,
        user_id: &str,
        username: &str,
        discriminator: Option<&str>,
        avatar: Option<&str>,
        guilds_json: &str,
        expires_at: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO sessions (token, user_id, username, discriminator, avatar, guilds_json, expires_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
             ON CONFLICT(token) DO UPDATE SET \
                user_id = excluded.user_id, \
                username = excluded.username, \
                discriminator = excluded.discriminator, \
                avatar = excluded.avatar, \
                guilds_json = excluded.guilds_json, \
                expires_at = excluded.expires_at",
        )
        .bind(token)
        .bind(user_id)
        .bind(username)
        .bind(discriminator)
        .bind(avatar)
        .bind(guilds_json)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn load_valid_sessions(&self) -> Result<Vec<SessionRow>, sqlx::Error> {
        sqlx::query_as::<_, SessionRow>(
            "SELECT token, user_id, username, discriminator, avatar, guilds_json, expires_at \
             FROM sessions WHERE expires_at > datetime('now')",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn remove_session(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_reaction_roles(&self, guild_id: &str) -> Result<Vec<ReactionRole>, sqlx::Error> {
        sqlx::query_as::<_, ReactionRole>(
            "SELECT id, guild_id, channel_id, message_id, role_id, emoji, created_at \
             FROM reaction_roles WHERE guild_id = ? ORDER BY id",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn add_reaction_role(&self, guild_id: &str, channel_id: &str, message_id: &str, role_id: &str, emoji: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR IGNORE INTO reaction_roles (guild_id, channel_id, message_id, role_id, emoji) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(guild_id)
        .bind(channel_id)
        .bind(message_id)
        .bind(role_id)
        .bind(emoji)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_reaction_role(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM reaction_roles WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_custom_command(&self, guild_id: &str, name: &str) -> Result<Option<CustomCommand>, sqlx::Error> {
        sqlx::query_as::<_, CustomCommand>(
            "SELECT id, guild_id, name, script, enabled, created_at, updated_at \
             FROM custom_commands WHERE guild_id = ?1 AND name = ?2",
        )
        .bind(guild_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_custom_commands(&self, guild_id: &str) -> Result<Vec<CustomCommand>, sqlx::Error> {
        sqlx::query_as::<_, CustomCommand>(
            "SELECT id, guild_id, name, script, enabled, created_at, updated_at \
             FROM custom_commands WHERE guild_id = ? ORDER BY name",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn set_custom_command(&self, guild_id: &str, name: &str, script: &str, enabled: bool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO custom_commands (guild_id, name, script, enabled, updated_at) \
             VALUES (?1, ?2, ?3, ?4, datetime('now')) \
             ON CONFLICT(guild_id, name) DO UPDATE SET \
                script = excluded.script, \
                enabled = excluded.enabled, \
                updated_at = datetime('now')",
        )
        .bind(guild_id)
        .bind(name)
        .bind(script)
        .bind(enabled)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_custom_command(&self, guild_id: &str, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM custom_commands WHERE guild_id = ?1 AND name = ?2")
            .bind(guild_id)
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_giveaway(
        &self, guild_id: &str, channel_id: &str, message_id: &str,
        prize: &str, winners_count: i64, end_time: &str,
    ) -> Result<Giveaway, sqlx::Error> {
        sqlx::query(
            "INSERT INTO giveaways (guild_id, channel_id, message_id, prize, winners_count, end_time) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(guild_id)
        .bind(channel_id)
        .bind(message_id)
        .bind(prize)
        .bind(winners_count)
        .bind(end_time)
        .execute(&self.pool)
        .await?;

        let g = sqlx::query_as::<_, Giveaway>(
            "SELECT id, guild_id, channel_id, message_id, prize, winners_count, \
                    end_time, ended, entries, created_at \
             FROM giveaways WHERE rowid = last_insert_rowid()",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(g)
    }

    pub async fn get_giveaway_by_message(
        &self, guild_id: &str, message_id: &str,
    ) -> Result<Option<Giveaway>, sqlx::Error> {
        sqlx::query_as::<_, Giveaway>(
            "SELECT id, guild_id, channel_id, message_id, prize, winners_count, \
                    end_time, ended, entries, created_at \
             FROM giveaways WHERE guild_id = ?1 AND message_id = ?2",
        )
        .bind(guild_id)
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_active_giveaways(&self) -> Result<Vec<Giveaway>, sqlx::Error> {
        sqlx::query_as::<_, Giveaway>(
            "SELECT id, guild_id, channel_id, message_id, prize, winners_count, \
                    end_time, ended, entries, created_at \
             FROM giveaways WHERE ended = 0 ORDER BY end_time",
        )
        .fetch_all(&self.pool)
        .await
    }

    #[allow(dead_code)]
    pub async fn list_guild_giveaways(
        &self, guild_id: &str,
    ) -> Result<Vec<Giveaway>, sqlx::Error> {
        sqlx::query_as::<_, Giveaway>(
            "SELECT id, guild_id, channel_id, message_id, prize, winners_count, \
                    end_time, ended, entries, created_at \
             FROM giveaways WHERE guild_id = ? ORDER BY created_at DESC",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_giveaway_entries(
        &self, guild_id: &str, message_id: &str, entries: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE giveaways SET entries = ?1 WHERE guild_id = ?2 AND message_id = ?3",
        )
        .bind(entries)
        .bind(guild_id)
        .bind(message_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn end_giveaway(
        &self, guild_id: &str, message_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE giveaways SET ended = 1 WHERE guild_id = ?1 AND message_id = ?2",
        )
        .bind(guild_id)
        .bind(message_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_ticket_config(&self, guild_id: &str) -> Result<Option<TicketConfig>, sqlx::Error> {
        sqlx::query_as::<_, TicketConfig>(
            "SELECT guild_id, category_id, staff_role_id, panel_channel_id, panel_message_id, \
                    enabled, updated_at \
             FROM ticket_config WHERE guild_id = ?",
        )
        .bind(guild_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn set_ticket_config(&self, config: &TicketConfig) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO ticket_config (guild_id, category_id, staff_role_id, panel_channel_id, \
                                        panel_message_id, enabled, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now')) \
             ON CONFLICT(guild_id) DO UPDATE SET \
                category_id = excluded.category_id, \
                staff_role_id = excluded.staff_role_id, \
                panel_channel_id = excluded.panel_channel_id, \
                panel_message_id = excluded.panel_message_id, \
                enabled = excluded.enabled, \
                updated_at = datetime('now')",
        )
        .bind(&config.guild_id)
        .bind(&config.category_id)
        .bind(&config.staff_role_id)
        .bind(&config.panel_channel_id)
        .bind(&config.panel_message_id)
        .bind(config.enabled)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn create_ticket(
        &self, guild_id: &str, channel_id: &str, creator_id: &str,
    ) -> Result<Ticket, sqlx::Error> {
        sqlx::query(
            "INSERT INTO tickets (guild_id, channel_id, creator_id) VALUES (?1, ?2, ?3)",
        )
        .bind(guild_id)
        .bind(channel_id)
        .bind(creator_id)
        .execute(&self.pool)
        .await?;

        let t = sqlx::query_as::<_, Ticket>(
            "SELECT id, guild_id, channel_id, creator_id, status, created_at, closed_at \
             FROM tickets WHERE rowid = last_insert_rowid()",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(t)
    }

    pub async fn get_ticket_by_channel(&self, guild_id: &str, channel_id: &str) -> Result<Option<Ticket>, sqlx::Error> {
        sqlx::query_as::<_, Ticket>(
            "SELECT id, guild_id, channel_id, creator_id, status, created_at, closed_at \
             FROM tickets WHERE guild_id = ?1 AND channel_id = ?2",
        )
        .bind(guild_id)
        .bind(channel_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_ticket_status(&self, channel_id: &str, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE tickets SET status = ?1, closed_at = CASE WHEN ?1 = 'closed' THEN datetime('now') ELSE closed_at END \
             WHERE channel_id = ?2",
        )
        .bind(status)
        .bind(channel_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_guild_tickets(&self, guild_id: &str) -> Result<Vec<Ticket>, sqlx::Error> {
        sqlx::query_as::<_, Ticket>(
            "SELECT id, guild_id, channel_id, creator_id, status, created_at, closed_at \
             FROM tickets WHERE guild_id = ? ORDER BY created_at DESC",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn reopen_ticket(&self, channel_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE tickets SET status = 'open', closed_at = NULL WHERE channel_id = ?1",
        )
        .bind(channel_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_xp_config(&self, guild_id: &str) -> Result<Option<XpConfig>, sqlx::Error> {
        sqlx::query_as::<_, XpConfig>(
            "SELECT guild_id, xp_per_message, cooldown_seconds, min_chars, \
                    voice_xp_enabled, voice_xp_interval_minutes \
             FROM xp_config WHERE guild_id = ?",
        )
        .bind(guild_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn set_xp_config(&self, config: &XpConfig) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO xp_config (guild_id, xp_per_message, cooldown_seconds, min_chars, \
                                    voice_xp_enabled, voice_xp_interval_minutes) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(guild_id) DO UPDATE SET \
                xp_per_message = excluded.xp_per_message, \
                cooldown_seconds = excluded.cooldown_seconds, \
                min_chars = excluded.min_chars, \
                voice_xp_enabled = excluded.voice_xp_enabled, \
                voice_xp_interval_minutes = excluded.voice_xp_interval_minutes",
        )
        .bind(&config.guild_id)
        .bind(config.xp_per_message)
        .bind(config.cooldown_seconds)
        .bind(config.min_chars)
        .bind(config.voice_xp_enabled)
        .bind(config.voice_xp_interval_minutes)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_xp_data(&self, guild_id: &str, user_id: &str) -> Result<Option<XpData>, sqlx::Error> {
        sqlx::query_as::<_, XpData>(
            "SELECT id, guild_id, user_id, xp, level, last_xp_time \
             FROM xp_data WHERE guild_id = ?1 AND user_id = ?2",
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn upsert_xp_data(&self, guild_id: &str, user_id: &str, xp: i64, level: i64, last_xp_time: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO xp_data (guild_id, user_id, xp, level, last_xp_time) \
             VALUES (?1, ?2, ?3, ?4, ?5) \
             ON CONFLICT(guild_id, user_id) DO UPDATE SET \
                xp = excluded.xp, \
                level = excluded.level, \
                last_xp_time = excluded.last_xp_time",
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(xp)
        .bind(level)
        .bind(last_xp_time)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_xp_leaderboard(&self, guild_id: &str, limit: i64) -> Result<Vec<XpData>, sqlx::Error> {
        sqlx::query_as::<_, XpData>(
            "SELECT id, guild_id, user_id, xp, level, last_xp_time \
             FROM xp_data WHERE guild_id = ? ORDER BY xp DESC LIMIT ?",
        )
        .bind(guild_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_xp_rewards(&self, guild_id: &str) -> Result<Vec<XpReward>, sqlx::Error> {
        sqlx::query_as::<_, XpReward>(
            "SELECT id, guild_id, level, role_id FROM xp_rewards WHERE guild_id = ? ORDER BY level ASC",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn add_xp_reward(&self, guild_id: &str, level: i64, role_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR IGNORE INTO xp_rewards (guild_id, level, role_id) VALUES (?1, ?2, ?3)",
        )
        .bind(guild_id)
        .bind(level)
        .bind(role_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_xp_reward(&self, guild_id: &str, level: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM xp_rewards WHERE guild_id = ?1 AND level = ?2")
            .bind(guild_id)
            .bind(level)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, FromRow, serde::Serialize, serde::Deserialize)]
pub struct TicketConfig {
    pub guild_id: String,
    pub category_id: String,
    pub staff_role_id: String,
    pub panel_channel_id: String,
    pub panel_message_id: String,
    pub enabled: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Ticket {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: String,
    pub creator_id: String,
    pub status: String,
    pub created_at: String,
    pub closed_at: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SessionRow {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub discriminator: Option<String>,
    pub avatar: Option<String>,
    pub guilds_json: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, FromRow)]
struct GuildConfigRow {
    guild_id: String,
    moderation_enabled: bool,
    auto_mod_enabled: bool,
    logging_enabled: bool,
    welcome_enabled: bool,
    custom_commands_enabled: bool,
    reaction_roles_enabled: bool,
    tickets_enabled: bool,
    xp_enabled: bool,
    scheduling_enabled: bool,
    auto_mod_config: String,
    welcome_config: String,
}

impl GuildConfigRow {
    fn into_config(self) -> GuildConfig {
        let auto_mod: AutoModConfig =
            serde_json::from_str(&self.auto_mod_config).unwrap_or_default();
        let welcome: crate::config::WelcomeConfig =
            serde_json::from_str(&self.welcome_config).unwrap_or_default();
        GuildConfig {
            guild_id: self.guild_id,
            modules: ModulesConfig {
                moderation: self.moderation_enabled,
                auto_mod: self.auto_mod_enabled,
                logging: self.logging_enabled,
                welcome: self.welcome_enabled,
                custom_commands: self.custom_commands_enabled,
                reaction_roles: self.reaction_roles_enabled,
                tickets: self.tickets_enabled,
                xp: self.xp_enabled,
                scheduling: self.scheduling_enabled,
            },
            auto_mod,
            welcome,
        }
    }
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ReactionRole {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: String,
    pub message_id: String,
    pub role_id: String,
    pub emoji: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct CustomCommand {
    pub id: i64,
    pub guild_id: String,
    pub name: String,
    pub script: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Warning {
    pub id: i64,
    pub guild_id: String,
    pub user_id: String,
    pub moderator_id: String,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Giveaway {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: String,
    pub message_id: String,
    pub prize: String,
    pub winners_count: i64,
    pub end_time: String,
    pub ended: bool,
    pub entries: String,
    pub created_at: String,
}

impl Giveaway {
    pub fn entries_vec(&self) -> Vec<String> {
        serde_json::from_str(&self.entries).unwrap_or_default()
    }

    pub fn add_entry(&mut self, user_id: &str) -> bool {
        let mut entries = self.entries_vec();
        if entries.iter().any(|e| e == user_id) {
            return false;
        }
        entries.push(user_id.to_string());
        self.entries = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string());
        true
    }

    pub fn remove_entry(&mut self, user_id: &str) -> bool {
        let mut entries = self.entries_vec();
        let len_before = entries.len();
        entries.retain(|e| e != user_id);
        if entries.len() == len_before {
            return false;
        }
        self.entries = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string());
        true
    }

    pub fn is_expired(&self) -> bool {
        chrono::DateTime::parse_from_rfc3339(&self.end_time)
            .map(|t| chrono::Utc::now() > t)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Default, FromRow, serde::Serialize, serde::Deserialize)]
pub struct XpConfig {
    pub guild_id: String,
    pub xp_per_message: i64,
    pub cooldown_seconds: i64,
    pub min_chars: i64,
    pub voice_xp_enabled: bool,
    pub voice_xp_interval_minutes: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct XpData {
    pub id: i64,
    pub guild_id: String,
    pub user_id: String,
    pub xp: i64,
    pub level: i64,
    pub last_xp_time: Option<String>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct XpReward {
    pub id: i64,
    pub guild_id: String,
    pub level: i64,
    pub role_id: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ModeratorRole {
    pub id: i64,
    pub guild_id: String,
    pub role_id: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct AfkStatus {
    pub user_id: String,
    pub guild_id: String,
    pub message: String,
    pub channel_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Reminder {
    pub id: i64,
    pub user_id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub message: String,
    pub remind_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct SelfRole {
    pub id: i64,
    pub guild_id: String,
    pub role_id: String,
    pub name: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ModNote {
    pub id: i64,
    pub guild_id: String,
    pub user_id: String,
    pub moderator_id: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ScheduledAnnouncement {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: String,
    pub title: String,
    pub message: String,
    pub interval_minutes: Option<i64>,
    pub next_run_at: String,
    pub enabled: bool,
    pub created_at: String,
    pub created_by: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ScheduledAction {
    pub id: i64,
    pub guild_id: String,
    pub user_id: String,
    pub action_type: String,
    pub execute_at: String,
    pub reason: Option<String>,
    pub executed: bool,
    pub created_at: String,
}

impl Database {
    pub async fn create_scheduled_announcement(
        &self, guild_id: &str, channel_id: &str, title: &str,
        message: &str, interval_minutes: Option<i64>, next_run_at: &str, created_by: &str,
    ) -> Result<ScheduledAnnouncement, sqlx::Error> {
        sqlx::query(
            "INSERT INTO scheduled_announcements (guild_id, channel_id, title, message, interval_minutes, next_run_at, created_by) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(guild_id)
        .bind(channel_id)
        .bind(title)
        .bind(message)
        .bind(interval_minutes)
        .bind(next_run_at)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        let a = sqlx::query_as::<_, ScheduledAnnouncement>(
            "SELECT id, guild_id, channel_id, title, message, interval_minutes, next_run_at, enabled, created_at, created_by \
             FROM scheduled_announcements WHERE rowid = last_insert_rowid()",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(a)
    }

    pub async fn list_scheduled_announcements(&self, guild_id: &str) -> Result<Vec<ScheduledAnnouncement>, sqlx::Error> {
        sqlx::query_as::<_, ScheduledAnnouncement>(
            "SELECT id, guild_id, channel_id, title, message, interval_minutes, next_run_at, enabled, created_at, created_by \
             FROM scheduled_announcements WHERE guild_id = ? ORDER BY next_run_at ASC",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_due_scheduled_announcements(&self) -> Result<Vec<ScheduledAnnouncement>, sqlx::Error> {
        sqlx::query_as::<_, ScheduledAnnouncement>(
            "SELECT id, guild_id, channel_id, title, message, interval_minutes, next_run_at, enabled, created_at, created_by \
             FROM scheduled_announcements WHERE enabled = 1 AND next_run_at <= datetime('now') ORDER BY next_run_at",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_announcement_next_run(&self, id: i64, next_run_at: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_announcements SET next_run_at = ?1 WHERE id = ?2")
            .bind(next_run_at)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn disable_scheduled_announcement(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_announcements SET enabled = 0 WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_scheduled_announcement(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM scheduled_announcements WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_scheduled_action(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM scheduled_actions WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_scheduled_action(
        &self, guild_id: &str, user_id: &str, action_type: &str,
        execute_at: &str, reason: Option<&str>,
    ) -> Result<ScheduledAction, sqlx::Error> {
        sqlx::query(
            "INSERT INTO scheduled_actions (guild_id, user_id, action_type, execute_at, reason) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(action_type)
        .bind(execute_at)
        .bind(reason)
        .execute(&self.pool)
        .await?;

        let a = sqlx::query_as::<_, ScheduledAction>(
            "SELECT id, guild_id, user_id, action_type, execute_at, reason, executed, created_at \
             FROM scheduled_actions WHERE rowid = last_insert_rowid()",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(a)
    }

    pub async fn get_due_scheduled_actions(&self) -> Result<Vec<ScheduledAction>, sqlx::Error> {
        sqlx::query_as::<_, ScheduledAction>(
            "SELECT id, guild_id, user_id, action_type, execute_at, reason, executed, created_at \
             FROM scheduled_actions WHERE executed = 0 AND execute_at <= datetime('now') ORDER BY execute_at",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn mark_scheduled_action_executed(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_actions SET executed = 1 WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_scheduled_actions(&self, guild_id: &str) -> Result<Vec<ScheduledAction>, sqlx::Error> {
        sqlx::query_as::<_, ScheduledAction>(
            "SELECT id, guild_id, user_id, action_type, execute_at, reason, executed, created_at \
             FROM scheduled_actions WHERE guild_id = ? ORDER BY execute_at ASC",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
    }
}

impl Database {
    // === Moderator Roles ===
    pub async fn add_moderator_role(&self, guild_id: &str, role_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR IGNORE INTO moderator_roles (guild_id, role_id) VALUES (?1, ?2)")
            .bind(guild_id).bind(role_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn remove_moderator_role(&self, guild_id: &str, role_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM moderator_roles WHERE guild_id = ?1 AND role_id = ?2")
            .bind(guild_id).bind(role_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_moderator_roles(&self, guild_id: &str) -> Result<Vec<ModeratorRole>, sqlx::Error> {
        sqlx::query_as::<_, ModeratorRole>(
            "SELECT id, guild_id, role_id FROM moderator_roles WHERE guild_id = ?"
        ).bind(guild_id).fetch_all(&self.pool).await
    }

    pub async fn is_moderator_role(&self, guild_id: &str, role_id: &str) -> Result<bool, sqlx::Error> {
        let row: Option<ModeratorRole> = sqlx::query_as::<_, ModeratorRole>(
            "SELECT id, guild_id, role_id FROM moderator_roles WHERE guild_id = ?1 AND role_id = ?2"
        ).bind(guild_id).bind(role_id).fetch_optional(&self.pool).await?;
        Ok(row.is_some())
    }

    // === AFK ===
    pub async fn set_afk(&self, user_id: &str, guild_id: &str, message: &str, channel_id: Option<&str>) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO afk_status (user_id, guild_id, message, channel_id, created_at) \
             VALUES (?1, ?2, ?3, ?4, datetime('now'))"
        ).bind(user_id).bind(guild_id).bind(message).bind(channel_id)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn remove_afk(&self, user_id: &str, guild_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM afk_status WHERE user_id = ?1 AND guild_id = ?2")
            .bind(user_id).bind(guild_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_afk(&self, user_id: &str, guild_id: &str) -> Result<Option<AfkStatus>, sqlx::Error> {
        sqlx::query_as::<_, AfkStatus>(
            "SELECT user_id, guild_id, message, channel_id, created_at FROM afk_status WHERE user_id = ?1 AND guild_id = ?2"
        ).bind(user_id).bind(guild_id).fetch_optional(&self.pool).await
    }

    pub async fn list_afk(&self, guild_id: &str) -> Result<Vec<AfkStatus>, sqlx::Error> {
        sqlx::query_as::<_, AfkStatus>(
            "SELECT user_id, guild_id, message, channel_id, created_at FROM afk_status WHERE guild_id = ?"
        ).bind(guild_id).fetch_all(&self.pool).await
    }

    // === Reminders ===
    pub async fn create_reminder(&self, user_id: &str, guild_id: &str, channel_id: &str, message: &str, remind_at: &str) -> Result<Reminder, sqlx::Error> {
        sqlx::query(
            "INSERT INTO reminders (user_id, guild_id, channel_id, message, remind_at) VALUES (?1, ?2, ?3, ?4, ?5)"
        ).bind(user_id).bind(guild_id).bind(channel_id).bind(message).bind(remind_at)
        .execute(&self.pool).await?;

        let r = sqlx::query_as::<_, Reminder>(
            "SELECT id, user_id, guild_id, channel_id, message, remind_at, created_at FROM reminders WHERE rowid = last_insert_rowid()"
        ).fetch_one(&self.pool).await?;
        Ok(r)
    }

    pub async fn get_due_reminders(&self) -> Result<Vec<Reminder>, sqlx::Error> {
        sqlx::query_as::<_, Reminder>(
            "SELECT id, user_id, guild_id, channel_id, message, remind_at, created_at FROM reminders WHERE remind_at <= datetime('now')"
        ).fetch_all(&self.pool).await
    }

    pub async fn delete_reminder(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM reminders WHERE id = ?1").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_reminders(&self, user_id: &str) -> Result<Vec<Reminder>, sqlx::Error> {
        sqlx::query_as::<_, Reminder>(
            "SELECT id, user_id, guild_id, channel_id, message, remind_at, created_at FROM reminders WHERE user_id = ? ORDER BY remind_at"
        ).bind(user_id).fetch_all(&self.pool).await
    }

    // === Self Roles (Ranks) ===
    pub async fn add_self_role(&self, guild_id: &str, name: &str, role_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR IGNORE INTO self_roles (guild_id, name, role_id) VALUES (?1, ?2, ?3)")
            .bind(guild_id).bind(name).bind(role_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn remove_self_role(&self, guild_id: &str, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM self_roles WHERE guild_id = ?1 AND name = ?2")
            .bind(guild_id).bind(name).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_self_roles(&self, guild_id: &str) -> Result<Vec<SelfRole>, sqlx::Error> {
        sqlx::query_as::<_, SelfRole>(
            "SELECT id, guild_id, role_id, name FROM self_roles WHERE guild_id = ? ORDER BY name"
        ).bind(guild_id).fetch_all(&self.pool).await
    }

    pub async fn get_self_role_by_name(&self, guild_id: &str, name: &str) -> Result<Option<SelfRole>, sqlx::Error> {
        sqlx::query_as::<_, SelfRole>(
            "SELECT id, guild_id, role_id, name FROM self_roles WHERE guild_id = ?1 AND name = ?2"
        ).bind(guild_id).bind(name).fetch_optional(&self.pool).await
    }

    // === Mod Notes ===
    pub async fn add_mod_note(&self, guild_id: &str, user_id: &str, moderator_id: &str, content: &str) -> Result<ModNote, sqlx::Error> {
        sqlx::query(
            "INSERT INTO mod_notes (guild_id, user_id, moderator_id, content) VALUES (?1, ?2, ?3, ?4)"
        ).bind(guild_id).bind(user_id).bind(moderator_id).bind(content)
        .execute(&self.pool).await?;

        let n = sqlx::query_as::<_, ModNote>(
            "SELECT id, guild_id, user_id, moderator_id, content, created_at FROM mod_notes WHERE rowid = last_insert_rowid()"
        ).fetch_one(&self.pool).await?;
        Ok(n)
    }

    pub async fn list_mod_notes(&self, guild_id: &str, user_id: &str) -> Result<Vec<ModNote>, sqlx::Error> {
        sqlx::query_as::<_, ModNote>(
            "SELECT id, guild_id, user_id, moderator_id, content, created_at FROM mod_notes WHERE guild_id = ?1 AND user_id = ?2 ORDER BY created_at DESC"
        ).bind(guild_id).bind(user_id).fetch_all(&self.pool).await
    }

    pub async fn delete_mod_note(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM mod_notes WHERE id = ?1").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn edit_mod_note(&self, id: i64, content: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE mod_notes SET content = ?1 WHERE id = ?2")
            .bind(content).bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn clear_user_warnings(&self, guild_id: &str, user_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM warnings WHERE guild_id = ?1 AND user_id = ?2")
            .bind(guild_id).bind(user_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_warning(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM warnings WHERE id = ?1").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn edit_warning(&self, id: i64, reason: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE warnings SET reason = ?1 WHERE id = ?2")
            .bind(reason).bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_warning_by_id(&self, id: i64) -> Result<Option<Warning>, sqlx::Error> {
        sqlx::query_as::<_, Warning>(
            "SELECT id, guild_id, user_id, moderator_id, reason, created_at FROM warnings WHERE id = ?"
        ).bind(id).fetch_optional(&self.pool).await
    }

    pub async fn get_warning_count(&self, guild_id: &str, user_id: &str) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM warnings WHERE guild_id = ?1 AND user_id = ?2"
        ).bind(guild_id).bind(user_id).fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    pub async fn get_total_warnings(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM warnings")
            .fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    pub async fn get_total_custom_commands(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM custom_commands")
            .fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    pub async fn get_total_giveaways(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM giveaways")
            .fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    pub async fn get_active_giveaway_count(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM giveaways WHERE ended = 0")
            .fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    pub async fn get_total_tickets(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tickets")
            .fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    pub async fn get_open_ticket_count(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tickets WHERE status = 'open'")
            .fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    pub async fn get_total_guild_configs(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM guild_configs")
            .fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    pub async fn get_total_reaction_roles(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM reaction_roles")
            .fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    pub async fn get_total_xp_data(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM xp_data")
            .fetch_one(&self.pool).await?;
        Ok(row.0)
    }
}

pub fn xp_level(xp: i64) -> i64 {
    (xp as f64 / 100.0).sqrt() as i64 + 1
}

pub fn xp_for_level(level: i64) -> i64 {
    if level <= 1 { 0 } else { (level - 1).pow(2) * 100 }
}

pub fn xp_to_next_level(xp: i64) -> i64 {
    let current_level = xp_level(xp);
    let current_min = xp_for_level(current_level);
    let next_min = xp_for_level(current_level + 1);
    let progress = xp - current_min;
    let needed = next_min - current_min;
    if needed == 0 { 100 } else { needed - progress }
}

pub fn xp_progress(xp: i64) -> f64 {
    let current_level = xp_level(xp);
    let current_min = xp_for_level(current_level);
    let next_min = xp_for_level(current_level + 1);
    let range = next_min - current_min;
    if range == 0 { 0.0 } else { (xp - current_min) as f64 / range as f64 }
}

fn resolve_sqlite_url(url: &str) -> String {
    if let Some(path) = url.strip_prefix("sqlite:") {
        let p = std::path::Path::new(path);
        if p.is_relative() {
            if let Ok(cwd) = std::env::current_dir() {
                let abs = cwd.join(path);
                return format!("sqlite:{}", abs.display());
            }
        }
    }
    url.to_string()
}
