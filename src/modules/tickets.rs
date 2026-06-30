use crate::database::TicketConfig;
use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;
use poise::Command;

pub fn commands() -> Vec<Command<AppState, Error>> {
    vec![ticket()]
}

#[poise::command(slash_command, subcommands("setup", "close", "claim", "add", "remove"))]
async fn ticket(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    ctx.say("Subcommands: `setup`, `close`, `claim`, `add`, `remove`").await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn setup(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Category ID to create ticket channels under"] category_id: String,
    #[description = "Staff role ID that can manage tickets"] staff_role_id: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let gid = guild_id.to_string();

    let _cat_id: u64 = category_id.trim().parse().map_err(|_| "Invalid category ID")?;
    let mut config = match ctx.data().db.get_ticket_config(&gid).await? {
        Some(c) => c,
        None => TicketConfig {
            guild_id: gid.clone(),
            category_id: category_id.clone(),
            staff_role_id: staff_role_id.clone().unwrap_or_default(),
            panel_channel_id: String::new(),
            panel_message_id: String::new(),
            enabled: true,
            updated_at: String::new(),
        },
    };

    config.category_id = category_id;
    if let Some(srid) = &staff_role_id {
        config.staff_role_id = srid.clone();
    }
    config.enabled = true;

    let embed = serenity::CreateEmbed::new()
        .title("🎫 Tickets")
        .description("React with 🎫 to create a ticket and get help from staff!")
        .color(serenity::Colour::BLUE);

    let msg = ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    msg.react(ctx, serenity::ReactionType::Unicode("🎫".to_string())).await?;

    config.panel_channel_id = ctx.channel_id().to_string();
    config.panel_message_id = msg.id.to_string();

    ctx.data().db.set_ticket_config(&config).await?;

    ctx.say("Ticket system set up! Panel created in this channel.").await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn close(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let channel_id = ctx.channel_id();
    let gid = guild_id.to_string();
    let cid = channel_id.to_string();

    let Some(ticket) = ctx.data().db.get_ticket_by_channel(&gid, &cid).await? else {
        return Err("This is not a ticket channel".into());
    };
    if ticket.status == "closed" {
        return Err("This ticket is already closed".into());
    }

    let config = ctx.data().db.get_ticket_config(&gid).await?.unwrap_or_default();
    let author_id = ctx.author().id.to_string();
    let is_creator = ticket.creator_id == author_id;
    let is_staff = if !config.staff_role_id.is_empty() {
        if let Ok(role_id) = config.staff_role_id.parse::<u64>() {
            ctx.author().has_role(ctx, guild_id, serenity::RoleId::new(role_id)).await.unwrap_or(false)
        } else { false }
    } else { false };

    if !is_creator && !is_staff {
        return Err("Only the ticket creator or staff can close this ticket".into());
    }

    let everyone = serenity::RoleId::new(guild_id.get());
    let overwrite = serenity::model::channel::PermissionOverwrite {
        allow: serenity::Permissions::empty(),
        deny: serenity::Permissions::SEND_MESSAGES,
        kind: serenity::model::channel::PermissionOverwriteType::Role(everyone),
    };

    let _ = channel_id.edit(ctx, serenity::builder::EditChannel::new().permissions(vec![overwrite])).await;

    ctx.data().db.update_ticket_status(&cid, "closed").await?;

    let _ = channel_id.send_message(ctx, serenity::CreateMessage::new().content("🔒 Ticket closed by **{user}**. This channel is now read-only.".replace("{user}", &ctx.author().name))).await;

    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES")]
async fn claim(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let channel_id = ctx.channel_id();
    let gid = guild_id.to_string();
    let cid = channel_id.to_string();

    let Some(ticket) = ctx.data().db.get_ticket_by_channel(&gid, &cid).await? else {
        return Err("This is not a ticket channel".into());
    };
    if ticket.status == "closed" {
        return Err("This ticket is closed".into());
    }
    if ticket.status == "claimed" {
        return Err("This ticket is already claimed".into());
    }

    ctx.data().db.update_ticket_status(&cid, "claimed").await?;

    let _ = channel_id.edit(ctx, serenity::builder::EditChannel::new().name(format!("claimed-{}", ticket.channel_id.chars().take(90).collect::<String>()))).await;
    let _ = channel_id.send_message(ctx, serenity::CreateMessage::new().content(format!("👋 **{}** claimed this ticket!", ctx.author().name))).await;

    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES")]
async fn add(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to add to the ticket"] user: serenity::User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let channel_id = ctx.channel_id();
    let gid = guild_id.to_string();
    let cid = channel_id.to_string();

    let Some(_ticket) = ctx.data().db.get_ticket_by_channel(&gid, &cid).await? else {
        return Err("This is not a ticket channel".into());
    };

    let overwrite = serenity::model::channel::PermissionOverwrite {
        allow: serenity::Permissions::VIEW_CHANNEL | serenity::Permissions::SEND_MESSAGES | serenity::Permissions::READ_MESSAGE_HISTORY,
        deny: serenity::Permissions::empty(),
        kind: serenity::model::channel::PermissionOverwriteType::Member(user.id),
    };
    channel_id.edit(ctx, serenity::builder::EditChannel::new().permissions(vec![overwrite])).await?;

    ctx.say(format!("Added {} to this ticket.", user.name)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES")]
async fn remove(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to remove from the ticket"] user: serenity::User,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id();

    let overwrite = serenity::model::channel::PermissionOverwrite {
        allow: serenity::Permissions::empty(),
        deny: serenity::Permissions::VIEW_CHANNEL,
        kind: serenity::model::channel::PermissionOverwriteType::Member(user.id),
    };
    channel_id.edit(ctx, serenity::builder::EditChannel::new().permissions(vec![overwrite])).await?;

    ctx.say(format!("Removed {} from this ticket.", user.name)).await?;
    Ok(())
}

pub async fn handle_ticket_reaction(
    ctx: &serenity::Context,
    reaction: &serenity::Reaction,
    state: &AppState,
) {
    let Some(guild_id) = reaction.guild_id else { return };
    let gid = guild_id.to_string();

    let Ok(Some(guild_config)) = state.db.get_guild_config(&gid).await else { return };
    if !guild_config.modules.tickets { return; }

    let config = match state.db.get_ticket_config(&gid).await {
        Ok(Some(c)) if c.enabled && c.panel_message_id == reaction.message_id.to_string() => c,
        _ => return,
    };

    let Some(user_id) = reaction.user_id else { return };
    if user_id == ctx.cache.current_user().id { return; }

    let channel_name = format!("ticket-{}", user_id);

    if let Ok(Some(_existing)) = state.db.get_ticket_by_channel(
        &gid, &format!("ticket-{}", user_id)
    ).await {
        let panel_ch = serenity::ChannelId::new(config.panel_channel_id.parse().unwrap_or(0));
        let _ = panel_ch.send_message(&ctx.http, serenity::CreateMessage::new().content(format!("<@{}> You already have an open ticket!", user_id))).await;
        return;
    }

    let cat_id: u64 = match config.category_id.parse() {
        Ok(id) => id,
        _ => return,
    };

    let everyone_id = serenity::RoleId::new(guild_id.get());

    let mut overwrites = vec![
        serenity::model::channel::PermissionOverwrite {
            allow: serenity::Permissions::VIEW_CHANNEL | serenity::Permissions::SEND_MESSAGES | serenity::Permissions::READ_MESSAGE_HISTORY,
            deny: serenity::Permissions::empty(),
            kind: serenity::model::channel::PermissionOverwriteType::Member(user_id),
        },
        serenity::model::channel::PermissionOverwrite {
            allow: serenity::Permissions::empty(),
            deny: serenity::Permissions::VIEW_CHANNEL,
            kind: serenity::model::channel::PermissionOverwriteType::Role(everyone_id),
        },
    ];

    if let Ok(staff_role_id) = config.staff_role_id.parse::<u64>() {
        overwrites.push(serenity::model::channel::PermissionOverwrite {
            allow: serenity::Permissions::VIEW_CHANNEL | serenity::Permissions::SEND_MESSAGES | serenity::Permissions::READ_MESSAGE_HISTORY | serenity::Permissions::MANAGE_MESSAGES,
            deny: serenity::Permissions::empty(),
            kind: serenity::model::channel::PermissionOverwriteType::Role(serenity::RoleId::new(staff_role_id)),
        });
    }

    let builder = serenity::builder::CreateChannel::new(&channel_name)
        .kind(serenity::model::channel::ChannelType::Text)
        .category(serenity::ChannelId::new(cat_id))
        .permissions(overwrites);

    let channel = match guild_id.create_channel(&ctx.http, builder).await {
        Ok(ch) => ch,
        Err(e) => {
            eprintln!("Failed to create ticket channel: {}", e);
            return;
        }
    };

    let ch_id = channel.id;

    let _ = state.db.create_ticket(&gid, &ch_id.to_string(), &user_id.to_string()).await;

    let welcome = serenity::CreateEmbed::new()
        .title("🎫 Ticket Created")
        .description(format!("Welcome <@{}>! Staff will be with you shortly.\nDescribe your issue and a team member will assist you.", user_id))
        .color(serenity::Colour::BLUE);

    let _ = ch_id.send_message(&ctx.http, serenity::CreateMessage::new().embed(welcome)).await;

    let panel_ch: serenity::ChannelId = config.panel_channel_id.parse().unwrap_or(serenity::ChannelId::new(0));
    let _ = panel_ch.send_message(&ctx.http, serenity::CreateMessage::new().content(format!("<@{}> created a ticket: <#{}>", user_id, ch_id))).await;
}
