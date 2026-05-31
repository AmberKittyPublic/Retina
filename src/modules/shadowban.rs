use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;
use poise::Command;
use serenity::builder::EditRole;
use serenity::model::channel::ChannelType;
use serenity::model::channel::PermissionOverwrite;
use serenity::model::channel::PermissionOverwriteType;
use serenity::model::Permissions;

pub const SHADOWBAN_ROLE_NAME: &str = "Shadow Banned";

pub fn commands() -> Vec<Command<AppState, Error>> {
    vec![shadowban(), shadowunban()]
}

pub async fn get_or_create_shadowban_role(
    ctx: poise::Context<'_, AppState, Error>,
    guild_id: serenity::GuildId,
) -> Result<(serenity::RoleId, bool), Error> {
    let existing = {
        let guild = ctx.cache().guild(guild_id);
        guild.and_then(|g| g.roles.values().find(|r| r.name == SHADOWBAN_ROLE_NAME).map(|r| r.id))
    };
    if let Some(role_id) = existing {
        return Ok((role_id, false));
    }

    let role = guild_id
        .create_role(ctx, EditRole::new()
            .name(SHADOWBAN_ROLE_NAME)
            .permissions(Permissions::empty())
            .colour(serenity::Colour::from_rgb(30, 30, 30))
            .hoist(false)
            .mentionable(false))
        .await?;

    for (channel_id, _channel) in &guild_id.channels(ctx).await? {
        let deny = Permissions::SEND_MESSAGES
            | Permissions::ADD_REACTIONS
            | Permissions::CREATE_PUBLIC_THREADS
            | Permissions::CREATE_PRIVATE_THREADS
            | Permissions::SEND_MESSAGES_IN_THREADS
            | Permissions::SPEAK
            | Permissions::STREAM
            | Permissions::USE_SOUNDBOARD
            | Permissions::USE_EMBEDDED_ACTIVITIES;
        let overwrite = PermissionOverwrite {
            allow: Permissions::empty(),
            deny,
            kind: PermissionOverwriteType::Role(role.id),
        };
        let _ = channel_id.create_permission(ctx, overwrite).await;
    }

    Ok((role.id, true))
}

#[poise::command(slash_command, required_permissions = "BAN_MEMBERS")]
async fn shadowban(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to shadowban"] user: serenity::User,
    #[description = "Reason for shadowban"] reason: Option<String>,
    #[description = "Whether to purge recent messages (default: true)"] purge: Option<bool>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let (role_id, _created) = get_or_create_shadowban_role(ctx, guild_id).await?;

    let member = guild_id.member(ctx, user.id).await?;
    if member.roles.contains(&role_id) {
        ctx.say(format!("{} is already shadowbanned.", user.name)).await?;
        return Ok(());
    }
    member.add_role(ctx, role_id).await?;

    let mut deleted_count = 0u64;
    if purge.unwrap_or(true) {
        for (channel_id, channel) in &guild_id.channels(ctx).await? {
            if channel.kind == ChannelType::Category || channel.kind == ChannelType::Voice || channel.kind == ChannelType::Stage {
                continue;
            }
            if let Ok(messages) = channel_id
                .messages(ctx, serenity::builder::GetMessages::new().limit(100))
                .await
            {
                let to_delete: Vec<serenity::MessageId> = messages
                    .iter()
                    .filter(|m| m.author.id == user.id)
                    .map(|m| m.id)
                    .collect();

                if !to_delete.is_empty() {
                    if to_delete.len() == 1 {
                        if channel_id.delete_message(ctx, to_delete[0]).await.is_ok() {
                            deleted_count += 1;
                        }
                    } else if channel_id.delete_messages(ctx, &to_delete).await.is_ok() {
                        deleted_count += to_delete.len() as u64;
                    }
                }
            }
        }
    }

    let embed = serenity::CreateEmbed::new()
        .title("User Shadow Banned")
        .field("User", user.name.clone(), true)
        .field("Moderator", ctx.author().name.clone(), true)
        .field("Messages Deleted", deleted_count.to_string(), true)
        .field("Reason", reason.clone().unwrap_or_else(|| "No reason provided".to_string()), false)
        .color(serenity::Colour::DARK_GREY);

    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "BAN_MEMBERS")]
async fn shadowunban(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to remove shadowban from"] user: serenity::User,
    #[description = "Reason for removing shadowban"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let role_id = {
        let guild = ctx.cache().guild(guild_id);
        guild.and_then(|g| g.roles.values().find(|r| r.name == SHADOWBAN_ROLE_NAME).map(|r| r.id))
    };
    let role_id = match role_id {
        Some(id) => id,
        None => {
            ctx.say("No shadowbanned users found (role does not exist).").await?;
            return Ok(());
        }
    };

    let member = guild_id.member(ctx, user.id).await?;
    if !member.roles.contains(&role_id) {
        ctx.say(format!("{} is not shadowbanned.", user.name)).await?;
        return Ok(());
    }

    member.remove_role(ctx, role_id).await?;

    let embed = serenity::CreateEmbed::new()
        .title("Shadow Ban Lifted")
        .field("User", user.name.clone(), true)
        .field("Moderator", ctx.author().name.clone(), true)
        .field("Reason", reason.clone().unwrap_or_else(|| "No reason provided".to_string()), false)
        .color(serenity::Colour::DARK_GREEN);

    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}
