use crate::database::{ScheduledAction, ScheduledAnnouncement};
use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;
use poise::Command;
use std::time::Duration;

pub fn commands() -> Vec<Command<AppState, Error>> {
    vec![tempban(), tempmute(), scheduled()]
}

// ── TEMPBAN ──

#[poise::command(slash_command, required_permissions = "BAN_MEMBERS")]
async fn tempban(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to temporarily ban"] user: serenity::User,
    #[description = "Duration in minutes"] duration: u32,
    #[description = "Reason for ban"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let reason_text = reason.unwrap_or_else(|| "No reason provided".to_string());

    guild_id.ban(ctx, user.id, 0).await?;

    let execute_at = chrono::Utc::now() + chrono::Duration::minutes(duration as i64);

    ctx.data().db.create_scheduled_action(
        &guild_id.to_string(),
        &user.id.to_string(),
        "unban",
        &execute_at.to_rfc3339(),
        Some(&reason_text),
    ).await?;

    let embed = serenity::CreateEmbed::new()
        .title("User Temp-Banned")
        .field("User", user.name.clone(), true)
        .field("Duration", format!("{} minutes", duration), true)
        .field("Reason", &reason_text, false)
        .field("Unban", format!("<t:{}:R>", execute_at.timestamp()), false)
        .color(serenity::Colour::RED);

    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

// ── TEMPMUTE ──

#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
async fn tempmute(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to timeout"] user: serenity::User,
    #[description = "Duration in minutes (max 40320 = 28 days)"] duration: u32,
    #[description = "Reason for timeout"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let mut member = guild_id.member(ctx, user.id).await?;
    let clamped = duration.min(40320);
    let until = chrono::Utc::now() + chrono::Duration::minutes(clamped as i64);
    member.disable_communication_until_datetime(ctx, until.into()).await?;

    let reason = reason.unwrap_or_else(|| "No reason provided".to_string());

    let embed = serenity::CreateEmbed::new()
        .title("User Timed Out")
        .field("User", user.name.clone(), true)
        .field("Duration", format!("{} minutes", clamped), true)
        .field("Reason", &reason, false)
        .field("Expires", format!("<t:{}:R>", until.timestamp()), false)
        .color(serenity::Colour::DARK_GREY);

    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

// ── SCHEDULED ──

#[poise::command(slash_command, subcommands("announce", "list", "cancel"))]
async fn scheduled(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    ctx.say("Subcommands: `announce`, `list`, `cancel`").await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn announce(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Channel to send the announcement"] channel: serenity::Channel,
    #[description = "Title of the announcement"] title: String,
    #[description = "Message content"] message: String,
    #[description = "Schedule time as YYYY-MM-DD HH:MM (UTC) or a timestamp"] schedule_at: String,
    #[description = "Repeat every N minutes (optional)"] interval_minutes: Option<i64>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let channel_id = match channel {
        serenity::Channel::Guild(ch) => ch.id,
        _ => return Err("Not a valid text channel".into()),
    };

    let next_run = parse_datetime(&schedule_at)?;

    ctx.data().db.create_scheduled_announcement(
        &guild_id.to_string(),
        &channel_id.to_string(),
        &title,
        &message,
        interval_minutes,
        &next_run.to_rfc3339(),
        &ctx.author().id.to_string(),
    ).await?;

    let repeat_text = if let Some(interval) = interval_minutes {
        format!(", repeating every {} minutes", interval)
    } else {
        String::new()
    };

    let embed = serenity::CreateEmbed::new()
        .title("Scheduled Announcement Created")
        .field("Title", &title, false)
        .field("Channel", format!("<#{}>", channel_id), false)
        .field("Scheduled", format!("<t:{}:F>{}", next_run.timestamp(), repeat_text), false)
        .color(serenity::Colour::BLUE);

    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn list(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let gid = guild_id.to_string();

    let announcements = ctx.data().db.list_scheduled_announcements(&gid).await?;
    let actions = ctx.data().db.list_scheduled_actions(&gid).await?;

    let mut lines = String::new();

    if announcements.is_empty() && actions.is_empty() {
        lines = "No scheduled items.".to_string();
    } else {
        if !announcements.is_empty() {
            lines.push_str("**📢 Scheduled Announcements:**\n");
            for a in &announcements {
                let status = if a.enabled { "🟢" } else { "🔴" };
                let interval = if let Some(m) = a.interval_minutes {
                    format!(" (every {}m)", m)
                } else {
                    String::new()
                };
                let ts = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&a.next_run_at) {
                    format!("<t:{}:R>", dt.timestamp())
                } else {
                    a.next_run_at.clone()
                };
                lines.push_str(&format!(
                    "`[A{}]` {} **{}** — {} — <#{}>{}\n",
                    a.id, status, a.title, ts, a.channel_id, interval
                ));
            }
        }

        if !actions.is_empty() {
            lines.push_str("\n**⏰ Scheduled Actions:**\n");
            for a in &actions {
                let done = if a.executed { "✅" } else { "⏳" };
                let ts = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&a.execute_at) {
                    format!("<t:{}:R>", dt.timestamp())
                } else {
                    a.execute_at.clone()
                };
                let label = match a.action_type.as_str() {
                    "unban" => "Unban",
                    _ => &a.action_type,
                };
                lines.push_str(&format!(
                    "`[S{}]` {} **{}** <@{}> — {}\n",
                    a.id, done, label, a.user_id, ts
                ));
            }
        }
    }

    let embed = serenity::CreateEmbed::new()
        .title("Scheduled Items")
        .description(lines)
        .color(serenity::Colour::BLUE);

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn cancel(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "ID from /scheduled list (prefix A for announcement, S for action)"] id: String,
) -> Result<(), Error> {
    if let Some(num) = id.strip_prefix('A') {
        let id: i64 = num.parse().map_err(|_| "Invalid announcement ID")?;
        ctx.data().db.delete_scheduled_announcement(id).await?;
        ctx.say(format!("Cancelled announcement `A{}`.", id)).await?;
    } else if let Some(num) = id.strip_prefix('S') {
        let id: i64 = num.parse().map_err(|_| "Invalid action ID")?;
        ctx.data().db.delete_scheduled_action(id).await?;
        ctx.say(format!("Cancelled action `S{}`.", id)).await?;
    } else {
        return Err("Prefix with A for announcement or S for action (e.g. A1)".into());
    }

    Ok(())
}

// ── BACKGROUND SCHEDULER ──

pub async fn run_scheduler(state: AppState) {
    let token = state.settings.discord.token.clone();
    let http = serenity::Http::new(&token);

    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;

        // Process due scheduled actions (unbans)
        if let Ok(actions) = state.db.get_due_scheduled_actions().await {
            for action in &actions {
                if let Err(e) = execute_action(&http, &state, action).await {
                    eprintln!("Failed to execute scheduled action {}: {}", action.id, e);
                }
                let _ = state.db.mark_scheduled_action_executed(action.id).await;
            }
        }

        // Process due scheduled announcements
        if let Ok(announcements) = state.db.get_due_scheduled_announcements().await {
            for ann in &announcements {
                if let Err(e) = send_announcement(&http, &state, ann).await {
                    eprintln!("Failed to send scheduled announcement {}: {}", ann.id, e);
                }

                if let Some(interval) = ann.interval_minutes {
                    let next = chrono::Utc::now() + chrono::Duration::minutes(interval);
                    let _ = state.db.update_announcement_next_run(ann.id, &next.to_rfc3339()).await;
                } else {
                    let _ = state.db.disable_scheduled_announcement(ann.id).await;
                }
            }
        }
    }
}

async fn execute_action(
    http: &serenity::Http,
    state: &AppState,
    action: &ScheduledAction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guild_id: serenity::GuildId = action.guild_id.parse().map_err(|_| "Invalid guild ID")?;
    let user_id: serenity::UserId = action.user_id.parse().map_err(|_| "Invalid user ID")?;

    match action.action_type.as_str() {
        "unban" => {
            guild_id.unban(http, user_id).await?;
            if let Some(ref ch) = find_mod_log_channel(state, &action.guild_id).await {
                let embed = serenity::CreateEmbed::new()
                    .title("User Unbanned (Scheduled)")
                    .field("User", format!("<@{}>", user_id), true)
                    .color(serenity::Colour::DARK_GREEN);
                let _ = ch.send_message(http, serenity::CreateMessage::new().embed(embed)).await;
            }
        }
        _ => {
            eprintln!("Unknown scheduled action type: {}", action.action_type);
        }
    }

    Ok(())
}

async fn send_announcement(
    http: &serenity::Http,
    _state: &AppState,
    ann: &ScheduledAnnouncement,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let channel_id: serenity::ChannelId = ann.channel_id.parse()?;

    let embed = serenity::CreateEmbed::new()
        .title(&ann.title)
        .description(&ann.message)
        .color(serenity::Colour::BLUE)
        .footer(serenity::CreateEmbedFooter::new("Scheduled Announcement"));

    channel_id.send_message(http, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

async fn find_mod_log_channel(state: &AppState, guild_id: &str) -> Option<serenity::ChannelId> {
    let config = state.db.get_guild_config(guild_id).await.ok()?;
    let config = config.as_ref()?;
    if !config.modules.logging {
        return None;
    }
    let guild_id_parsed: u64 = guild_id.parse().ok()?;
    let gid = serenity::GuildId::new(guild_id_parsed);
    let http = serenity::Http::new(&state.settings.discord.token);
    let channels = gid.channels(&http).await.ok()?;
    for (id, ch) in &channels {
        if ch.name == "mod-logs" {
            return Some(*id);
        }
    }
    None
}

fn parse_datetime(input: &str) -> Result<chrono::DateTime<chrono::Utc>, Error> {
    // Try RFC 3339 / ISO 8601 first
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(input) {
        return Ok(dt.with_timezone(&chrono::Utc));
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", input)) {
        return Ok(dt.with_timezone(&chrono::Utc));
    }

    // Try YYYY-MM-DD HH:MM format (assume UTC)
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M") {
        return Ok(dt.and_utc());
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt.and_utc());
    }

    Err("Invalid datetime format. Use YYYY-MM-DD HH:MM (UTC) or RFC 3339".into())
}
