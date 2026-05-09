use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command)]
pub async fn ping(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let start = std::time::Instant::now();
    let msg = ctx.say("Pong!").await?;
    let duration = start.elapsed();

    let mut state = ctx.data().bot_state.write().await;
    state.commands_executed += 1;

    msg.edit(ctx, poise::CreateReply::default().content(&format!("Pong! ({}ms)", duration.as_millis()))).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn info(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let state = ctx.data().bot_state.read().await;

    let embed = serenity::CreateEmbed::new()
        .title("Bot Information")
        .field("Commands Executed", state.commands_executed.to_string(), true)
        .field("Guild Count", ctx.cache().guilds().len().to_string(), true)
        .color(serenity::Colour::BLUE);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn stats(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let state = ctx.data().bot_state.read().await;

    let uptime = state.started_at
        .map(|t| t.elapsed().unwrap_or_default())
        .unwrap_or_default();
    let uptime_secs = uptime.as_secs();
    let days = uptime_secs / 86400;
    let hours = (uptime_secs % 86400) / 3600;
    let mins = (uptime_secs % 3600) / 60;
    let secs = uptime_secs % 60;

    let uptime_str = if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, mins, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else {
        format!("{}m {}s", mins, secs)
    };

    drop(state);

    let db = &ctx.data().db;
    let total_warnings = db.get_total_warnings().await.unwrap_or(0);
    let total_custom_commands = db.get_total_custom_commands().await.unwrap_or(0);
    let total_giveaways = db.get_total_giveaways().await.unwrap_or(0);
    let active_giveaways = db.get_active_giveaway_count().await.unwrap_or(0);
    let total_tickets = db.get_total_tickets().await.unwrap_or(0);
    let open_tickets = db.get_open_ticket_count().await.unwrap_or(0);
    let total_guild_configs = db.get_total_guild_configs().await.unwrap_or(0);
    let total_reaction_roles = db.get_total_reaction_roles().await.unwrap_or(0);
    let total_xp_users = db.get_total_xp_data().await.unwrap_or(0);

    let guild_count = ctx.cache().guilds().len();
    let bot_state = ctx.data().bot_state.read().await;

    let embed = serenity::CreateEmbed::new()
        .title("Bot Statistics")
        .field("Guilds", guild_count.to_string(), true)
        .field("Configured Guilds", total_guild_configs.to_string(), true)
        .field("Commands Executed", bot_state.commands_executed.to_string(), true)
        .field("Uptime", &uptime_str, true)
        .field("", "\u{200B}", false)
        .field("Total Warnings", total_warnings.to_string(), true)
        .field("Custom Commands", total_custom_commands.to_string(), true)
        .field("Giveaways", format!("{} active / {} total", active_giveaways, total_giveaways), true)
        .field("Tickets", format!("{} open / {} total", open_tickets, total_tickets), true)
        .field("Reaction Roles", total_reaction_roles.to_string(), true)
        .field("XP Tracked Users", total_xp_users.to_string(), true)
        .color(serenity::Colour::DARK_GREEN);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
