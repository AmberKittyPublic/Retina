pub mod extended;

use crate::types::AppState;
use crate::types::Error;
use poise::serenity_prelude as serenity;
use poise::{Command, Context};
use serenity::builder::{EditChannel, GetMessages};
use serenity::model::id::RoleId;
use serenity::model::Permissions;

pub struct ModerationModule;

impl ModerationModule {
    pub fn new() -> Self {
        ModerationModule
    }

    pub async fn handle_message(
        &self,
        _ctx: &serenity::Context,
        msg: &serenity::Message,
        state: &AppState,
    ) -> Result<(), Error> {
        if let Some(guild_id) = msg.guild_id {
            if let Ok(Some(config)) = state.db.get_guild_config(&guild_id.to_string()).await {
                if !config.modules.moderation {
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}

pub fn commands() -> Vec<Command<AppState, Error>> {
    let mut cmds = vec![ban(), kick(), warn(), warnings(), mute(), purge(), slowmode(), lockdown()];
    cmds.extend(extended::commands());
    cmds
}

#[poise::command(slash_command, required_permissions = "BAN_MEMBERS")]
async fn ban(
    ctx: Context<'_, AppState, Error>,
    #[description = "User to ban"] user: serenity::User,
    #[description = "Reason for ban"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    guild_id.ban(ctx, user.id, 0).await?;

    let embed = serenity::CreateEmbed::new()
        .title("User Banned")
        .field("User", user.name.clone(), true)
        .field("Moderator", ctx.author().name.clone(), true)
        .field("Reason", reason.clone().unwrap_or_else(|| "No reason provided".to_string()), false)
        .color(serenity::Colour::RED);

    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "KICK_MEMBERS")]
async fn kick(
    ctx: Context<'_, AppState, Error>,
    #[description = "User to kick"] user: serenity::User,
    #[description = "Reason for kick"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    guild_id.kick(ctx, user.id).await?;

    let embed = serenity::CreateEmbed::new()
        .title("User Kicked")
        .field("User", user.name.clone(), true)
        .field("Moderator", ctx.author().name.clone(), true)
        .field("Reason", reason.clone().unwrap_or_else(|| "No reason provided".to_string()), false)
        .color(serenity::Colour::ORANGE);

    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
async fn warn(
    ctx: Context<'_, AppState, Error>,
    #[description = "User to warn"] user: serenity::User,
    #[description = "Reason for warning"] reason: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    ctx.data().db.add_warning(
        &guild_id.to_string(),
        &user.id.to_string(),
        &ctx.author().id.to_string(),
        &reason,
    ).await?;

    let total = ctx.data().db.get_warnings(
        &guild_id.to_string(),
        &user.id.to_string(),
    ).await?.len();

    let embed = serenity::CreateEmbed::new()
        .title("User Warned")
        .field("User", user.name.clone(), true)
        .field("Moderator", ctx.author().name.clone(), true)
        .field("Reason", reason, false)
        .field("Total Warnings", total.to_string(), true)
        .color(serenity::Colour::ORANGE);

    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn warnings(
    ctx: Context<'_, AppState, Error>,
    #[description = "User to check warnings"] user: serenity::User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let warns = ctx.data().db.get_warnings(
        &guild_id.to_string(),
        &user.id.to_string(),
    ).await?;

    let count = warns.len();

    let details = if warns.is_empty() {
        "No warnings.".to_string()
    } else {
        warns.iter().enumerate().map(|(i, w)| {
            format!("{}. {} — <t:{}:f>", i + 1, w.reason, timestamp(&w.created_at))
        }).collect::<Vec<_>>().join("\n")
    };

    let embed = serenity::CreateEmbed::new()
        .title("User Warnings")
        .field("User", user.name.clone(), true)
        .field("Warnings", count.to_string(), true)
        .field("Details", &details, false)
        .color(if count == 0 { serenity::Colour::DARK_GREY } else { serenity::Colour::ORANGE });

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
async fn mute(
    ctx: Context<'_, AppState, Error>,
    #[description = "User to timeout"] user: serenity::User,
    #[description = "Duration in minutes (max 40320 = 28 days)"] duration: u32,
    #[description = "Reason for timeout"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let mut member = guild_id.member(ctx, user.id).await?;
    let until = chrono::Utc::now() + chrono::Duration::minutes(duration.min(40320) as i64);
    member.disable_communication_until_datetime(ctx, until.into()).await?;

    let embed = serenity::CreateEmbed::new()
        .title("User Timed Out")
        .field("User", user.name.clone(), true)
        .field("Duration", format!("{} minutes", duration.min(40320)), true)
        .field("Moderator", ctx.author().name.clone(), true)
        .field("Reason", reason.clone().unwrap_or_else(|| "No reason provided".to_string()), false)
        .color(serenity::Colour::DARK_GREY);

    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES")]
async fn purge(
    ctx: Context<'_, AppState, Error>,
    #[description = "Number of messages to delete (2-100)"] count: u32,
) -> Result<(), Error> {
    if count < 2 || count > 100 {
        return Err("Count must be between 2 and 100".into());
    }

    let channel_id = ctx.channel_id();
    let messages = channel_id.messages(ctx, GetMessages::new().limit(count as u8)).await?;
    let ids: Vec<_> = messages.iter().map(|m| m.id).collect();

    if ids.len() == 1 {
        channel_id.delete_message(ctx, ids[0]).await?;
    } else if ids.len() > 1 {
        channel_id.delete_messages(ctx, &ids).await?;
    }

    let embed = serenity::CreateEmbed::new()
        .title("Messages Purged")
        .field("Amount", format!("{} messages deleted", ids.len()), false)
        .field("Channel", format!("<#{}>", channel_id), false)
        .color(serenity::Colour::DARK_GREEN);

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_CHANNELS")]
async fn slowmode(
    ctx: Context<'_, AppState, Error>,
    #[description = "Seconds between messages (0 to disable, max 21600)"] seconds: u16,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id();
    let builder = EditChannel::new().rate_limit_per_user(seconds.min(21600));
    channel_id.edit(ctx, builder).await?;

    let embed = serenity::CreateEmbed::new()
        .title("Slowmode Set")
        .field("Channel", format!("<#{}>", channel_id), false)
        .field("Delay", format!("{} seconds", seconds.min(21600)), false)
        .color(serenity::Colour::BLUE);

    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_CHANNELS")]
async fn lockdown(
    ctx: Context<'_, AppState, Error>,
    #[description = "Lock or unlock the channel"] action: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let channel_id = ctx.channel_id();

    let everyone_role = RoleId::new(guild_id.get());

    match action.to_lowercase().as_str() {
        "lock" => {
            let overwrite = serenity::model::channel::PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::SEND_MESSAGES,
                kind: serenity::model::channel::PermissionOverwriteType::Role(everyone_role),
            };
            channel_id.edit(ctx, EditChannel::new().permissions(vec![overwrite])).await?;

            let embed = serenity::CreateEmbed::new()
                .title("Channel Locked")
                .field("Channel", format!("<#{}>", channel_id), false)
                .color(serenity::Colour::RED);
            ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
        }
        "unlock" => {
            let overwrite = serenity::model::channel::PermissionOverwrite {
                allow: Permissions::SEND_MESSAGES,
                deny: Permissions::empty(),
                kind: serenity::model::channel::PermissionOverwriteType::Role(everyone_role),
            };
            channel_id.edit(ctx, EditChannel::new().permissions(vec![overwrite])).await?;

            let embed = serenity::CreateEmbed::new()
                .title("Channel Unlocked")
                .field("Channel", format!("<#{}>", channel_id), false)
                .color(serenity::Colour::DARK_GREEN);
            ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
        }
        _ => {
            return Err("Action must be 'lock' or 'unlock'".into());
        }
    }

    Ok(())
}

fn timestamp(iso: &str) -> String {
    if let Ok(dt) = iso.parse::<chrono::NaiveDateTime>() {
        dt.and_utc().timestamp().to_string()
    } else {
        "unknown".to_string()
    }
}
