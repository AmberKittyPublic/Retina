use poise::serenity_prelude as serenity;
use serenity::model::id::GuildId;

pub struct Logging;

impl Logging {
    pub fn new() -> Self {
        Logging
    }

    pub async fn log_message_delete(
        &self,
        ctx: &serenity::Context,
        guild_id: GuildId,
        channel_id: serenity::ChannelId,
        message_id: serenity::MessageId,
    ) {
        let author_name = ctx.cache.message(channel_id, message_id)
            .map(|m| m.author.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let details = format!(
            "Channel: <#{}>\nAuthor: {}\nMessage ID: {}",
            channel_id, author_name, message_id
        );
        self.send_log(ctx, guild_id, "🗑️ Message Deleted", &details).await;
    }

    pub async fn log_message_edit(
        &self,
        ctx: &serenity::Context,
        guild_id: GuildId,
        channel_id: serenity::ChannelId,
        old_content: Option<String>,
        new_content: Option<String>,
        author: Option<String>,
    ) {
        if old_content == new_content {
            return;
        }

        let author_name = author.unwrap_or_else(|| "Unknown".to_string());
        let old = old_content.unwrap_or_else(|| "*not cached*".to_string());
        let new = new_content.unwrap_or_else(|| "*not cached*".to_string());

        let details = format!(
            "Channel: <#{}>\nAuthor: {}\n\n**Before:**\n{}\n\n**After:**\n{}",
            channel_id, author_name, old, new
        );
        self.send_log(ctx, guild_id, "✏️ Message Edited", &details).await;
    }

    pub async fn log_member_join(
        &self,
        ctx: &serenity::Context,
        guild_id: GuildId,
        member: &serenity::Member,
    ) {
        let created = timestamp_rfc3339(&member.user.created_at());
        let age = days_ago(&member.user.created_at());

        let details = format!(
            "User: {} (<@{}>)\nID: {}\nAccount Created: {} ({} days ago)",
            member.user.name, member.user.id, member.user.id, created, age
        );
        self.send_log(ctx, guild_id, "🟢 Member Joined", &details).await;
    }

    pub async fn log_member_leave(
        &self,
        ctx: &serenity::Context,
        guild_id: GuildId,
        user: &serenity::User,
        roles: &[serenity::Role],
    ) {
        let role_names: Vec<&str> = roles.iter().map(|r| r.name.as_str()).collect();
        let role_str = if role_names.is_empty() {
            "None".to_string()
        } else {
            role_names.join(", ")
        };

        let details = format!(
            "User: {} (<@{}>)\nID: {}\nRoles: {}",
            user.name, user.id, user.id, role_str
        );
        self.send_log(ctx, guild_id, "🔴 Member Left", &details).await;
    }

    pub async fn log_member_update(
        &self,
        ctx: &serenity::Context,
        guild_id: GuildId,
        event: &serenity::GuildMemberUpdateEvent,
    ) {
        let mut changes = Vec::new();

        if let Some(ref nick) = event.nick {
            changes.push(format!("Nickname set to `{}`", nick));
        }

        if !event.roles.is_empty() {
            let role_mentions: Vec<String> = event.roles.iter()
                .map(|r| format!("<@&{}>", r)).collect();
            changes.push(format!("Roles updated: {}", role_mentions.join(", ")));
        }

        if event.communication_disabled_until.is_some() {
            changes.push("Timed out".to_string());
        }

        if changes.is_empty() {
            return;
        }

        let details = format!(
            "User: {} (<@{}>)\nID: {}\n\n{}",
            event.user.name, event.user.id, event.user.id,
            changes.join("\n")
        );
        self.send_log(ctx, guild_id, "👤 Member Updated", &details).await;
    }

    pub async fn log_channel_create(
        &self,
        ctx: &serenity::Context,
        guild_id: GuildId,
        channel: &serenity::GuildChannel,
    ) {
        let details = format!(
            "Channel: <#{}>\nName: `{}`\nType: {}",
            channel.id, channel.name, channel.kind.name()
        );
        self.send_log(ctx, guild_id, "📝 Channel Created", &details).await;
    }

    pub async fn log_channel_delete(
        &self,
        ctx: &serenity::Context,
        guild_id: GuildId,
        channel: &serenity::GuildChannel,
    ) {
        let details = format!(
            "Name: `{}`\nType: {}",
            channel.name, channel.kind.name()
        );
        self.send_log(ctx, guild_id, "🗑️ Channel Deleted", &details).await;
    }

    pub async fn log_voice_state(
        &self,
        ctx: &serenity::Context,
        guild_id: GuildId,
        old: Option<&serenity::VoiceState>,
        new: &serenity::VoiceState,
    ) {
        let user_name = new.member.as_ref()
            .map(|m| m.user.name.clone())
            .unwrap_or_else(|| format!("<@{}>", new.user_id));

        let old_ch = old.and_then(|o| o.channel_id);
        let new_ch = new.channel_id;

        let details = match (old_ch, new_ch) {
            (None, Some(cid)) => {
                format!("User: {}\nJoined: <#{}>", user_name, cid)
            }
            (Some(_), None) => {
                format!("User: {}\nLeft: <#{}>", user_name, old_ch.unwrap())
            }
            (Some(o), Some(n)) if o != n => {
                format!("User: {}\nMoved: <#{}> → <#{}>", user_name, o, n)
            }
            _ => return,
        };

        self.send_log(ctx, guild_id, "🔊 Voice State Update", &details).await;
    }

    async fn send_log(
        &self,
        ctx: &serenity::Context,
        guild_id: GuildId,
        title: &str,
        details: &str,
    ) {
        if let Some(channel_id) = self.get_log_channel(ctx, guild_id).await {
            let embed = serenity::CreateEmbed::new()
                .title(title)
                .description(details)
                .timestamp(serenity::Timestamp::now())
                .color(serenity::Colour::BLUE);

            if let Err(e) = channel_id.send_message(&ctx.http, serenity::CreateMessage::new().embed(embed)).await {
                eprintln!("Failed to send log: {}", e);
            }
        }
    }

    async fn get_log_channel(
        &self,
        ctx: &serenity::Context,
        guild_id: GuildId,
    ) -> Option<serenity::ChannelId> {
        let guild = ctx.cache.guild(guild_id)?;
        guild.channels.iter()
            .find(|(_, ch)| ch.name == "mod-logs")
            .map(|(id, _)| *id)
    }
}

fn timestamp_rfc3339(ts: &serenity::Timestamp) -> String {
    ts.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

fn days_ago(ts: &serenity::Timestamp) -> i64 {
    let secs = ts.timestamp();
    let now = chrono::Utc::now().timestamp();
    (now - secs) / 86400
}
