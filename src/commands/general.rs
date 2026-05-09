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
    
    let embed = serenity::CreateEmbed::new()
        .title("Bot Statistics")
        .field("Uptime", format!("{:.2}s", uptime.as_secs_f64()), true)
        .field("Commands", state.commands_executed.to_string(), true)
        .color(serenity::Colour::DARK_GREEN);
    
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
