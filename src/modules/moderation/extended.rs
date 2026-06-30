use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;
use poise::Command;

pub fn commands() -> Vec<Command<AppState, Error>> {
    vec![
        softban(),
        members(),
        move_(),
        voicekick(),
        deafen(),
        vmute(),
        reason(),
        case(),
        notes(),
        clearwarn(),
        delwarn(),
    ]
}

#[poise::command(slash_command, required_permissions = "BAN_MEMBERS")]
async fn softban(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to softban"] user: serenity::User,
    #[description = "Reason for softban"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    guild_id.ban(ctx, user.id, 1).await?;
    guild_id.unban(ctx, user.id).await?;

    let embed = serenity::CreateEmbed::new()
        .title("User Soft-Banned")
        .field("User", user.name.clone(), true)
        .field("Moderator", ctx.author().name.clone(), true)
        .field("Reason", reason.clone().unwrap_or_else(|| "No reason provided".to_string()), false)
        .color(serenity::Colour::RED);
    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn members(
    ctx: poise::Context<'_, AppState, Error>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let (total, humans) = {
        let guild = ctx.cache().guild(guild_id).ok_or("Guild not found")?;
        let total = guild.member_count;
        let humans = guild.members.values().filter(|m| !m.user.bot).count() as u64;
        (total, humans)
    };
    let bots = total.saturating_sub(humans);
    let embed = serenity::CreateEmbed::new()
        .title("Member Statistics")
        .field("Total", total.to_string(), true)
        .field("Humans", humans.to_string(), true)
        .field("Bots", bots.to_string(), true)
        .color(serenity::Colour::DARK_GREEN);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, rename = "move", required_permissions = "MOVE_MEMBERS")]
async fn move_(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to move"] user: serenity::User,
    #[description = "Target voice channel"] channel: serenity::Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let member = guild_id.member(ctx, user.id).await?;
    let channel_id = channel.id();
    member.move_to_voice_channel(ctx, channel_id).await?;
    ctx.say(format!("Moved {} to <#{}>.", user.name, channel_id)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MOVE_MEMBERS")]
async fn voicekick(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to disconnect from voice"] user: serenity::User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let member = guild_id.member(ctx, user.id).await?;
    member.disconnect_from_voice(ctx).await?;
    ctx.say(format!("Disconnected {} from voice.", user.name)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "DEAFEN_MEMBERS")]
async fn deafen(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to deafen/undeafen"] user: serenity::User,
    #[description = "True to deafen, false to undeafen"] deaf: Option<bool>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let mut member = guild_id.member(ctx, user.id).await?;
    let deaf_state = deaf.unwrap_or(true);
    member.edit(ctx, serenity::EditMember::new().deafen(deaf_state)).await?;
    ctx.say(format!("{} {}.", if deaf_state { "Deafened" } else { "Undeafened" }, user.name)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MUTE_MEMBERS")]
async fn vmute(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to server mute/unmute"] user: serenity::User,
    #[description = "True to mute, false to unmute"] mute: Option<bool>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let mut member = guild_id.member(ctx, user.id).await?;
    let mute_state = mute.unwrap_or(true);
    member.edit(ctx, serenity::EditMember::new().mute(mute_state)).await?;
    ctx.say(format!("{} {}.", if mute_state { "Server muted" } else { "Unmuted" }, user.name)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
async fn reason(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Warning ID"] warning_id: i64,
    #[description = "New reason"] new_reason: String,
) -> Result<(), Error> {
    ctx.data().db.edit_warning(warning_id, &new_reason).await?;
    ctx.say(format!("Updated warning `{}` reason.", warning_id)).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn case(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Warning ID"] warning_id: i64,
) -> Result<(), Error> {
    let warning = ctx.data().db.get_warning_by_id(warning_id).await?
        .ok_or_else(|| format!("Warning `{}` not found.", warning_id))?;
    let embed = serenity::CreateEmbed::new()
        .title(format!("Case #{}", warning.id))
        .field("User", format!("<@{}>", warning.user_id), true)
        .field("Moderator", format!("<@{}>", warning.moderator_id), true)
        .field("Reason", &warning.reason, false)
        .field("Date", format!("<t:{}:f>", timestamp(&warning.created_at)), false)
        .color(serenity::Colour::ORANGE);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, subcommands("notes_add", "notes_list", "notes_delete", "notes_edit"))]
async fn notes(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    ctx.say("Subcommands: `add`, `list`, `delete`, `edit`").await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
async fn notes_add(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User"] user: serenity::User,
    #[description = "Note content"] content: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    ctx.data().db.add_mod_note(
        &guild_id.to_string(),
        &user.id.to_string(),
        &ctx.author().id.to_string(),
        &content,
    ).await?;
    ctx.say(format!("Added note for {} (ID: {}...).", user.name, &content[..content.len().min(20)])).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn notes_list(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User"] user: serenity::User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let notes = ctx.data().db.list_mod_notes(&guild_id.to_string(), &user.id.to_string()).await?;
    if notes.is_empty() {
        ctx.say(format!("No notes for {}.", user.name)).await?;
        return Ok(());
    }
    let desc: String = notes.iter().map(|n| {
        format!("`[{}]` {} — <@{}> — <t:{}:f>", n.id, n.content, n.moderator_id, timestamp(&n.created_at))
    }).collect::<Vec<_>>().join("\n");
    let embed = serenity::CreateEmbed::new()
        .title(format!("Notes for {}", user.name))
        .description(desc)
        .color(serenity::Colour::BLUE);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
async fn notes_delete(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Note ID"] note_id: i64,
) -> Result<(), Error> {
    ctx.data().db.delete_mod_note(note_id).await?;
    ctx.say(format!("Deleted note `{}`.", note_id)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
async fn notes_edit(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Note ID"] note_id: i64,
    #[description = "New content"] content: String,
) -> Result<(), Error> {
    ctx.data().db.edit_mod_note(note_id, &content).await?;
    ctx.say(format!("Edited note `{}`.", note_id)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
async fn clearwarn(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to clear warnings for"] user: serenity::User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    ctx.data().db.clear_user_warnings(&guild_id.to_string(), &user.id.to_string()).await?;
    ctx.say(format!("Cleared all warnings for {}.", user.name)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
async fn delwarn(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Warning ID"] warning_id: i64,
) -> Result<(), Error> {
    ctx.data().db.delete_warning(warning_id).await?;
    ctx.say(format!("Deleted warning `{}`.", warning_id)).await?;
    Ok(())
}

fn timestamp(iso: &str) -> String {
    if let Ok(dt) = iso.parse::<chrono::NaiveDateTime>() {
        dt.and_utc().timestamp().to_string()
    } else {
        "unknown".to_string()
    }
}
