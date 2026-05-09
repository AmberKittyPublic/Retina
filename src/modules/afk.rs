use crate::types::AppState;
use poise::serenity_prelude as serenity;

pub struct AfkModule;

impl AfkModule {
    pub fn new() -> Self { AfkModule }
}

pub fn commands() -> Vec<poise::Command<AppState, crate::types::Error>> {
    vec![afk(), afk_list()]
}

#[poise::command(slash_command)]
async fn afk(
    ctx: poise::Context<'_, AppState, crate::types::Error>,
    #[description = "AFK message"] message: Option<String>,
) -> Result<(), crate::types::Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let msg = message.as_deref().unwrap_or("AFK");
    let channel_id = if cfg!(feature = "afk_channel") { Some(ctx.channel_id().to_string()) } else { None };
    ctx.data().db.set_afk(
        &ctx.author().id.to_string(),
        &guild_id.to_string(),
        msg,
        channel_id.as_deref(),
    ).await?;
    let display = if msg == "AFK" { String::new() } else { format!(" — {}", msg) };
    ctx.say(format!("{} is now AFK{}", ctx.author().name, display)).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn afk_list(
    ctx: poise::Context<'_, AppState, crate::types::Error>,
) -> Result<(), crate::types::Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let users = ctx.data().db.list_afk(&guild_id.to_string()).await?;
    if users.is_empty() {
        ctx.say("No one is AFK.").await?;
        return Ok(());
    }
    let list: String = users.iter().map(|u| {
        format!("<@{}> — {}", u.user_id, u.message)
    }).collect::<Vec<_>>().join("\n");
    ctx.say(format!("**AFK Users:**\n{}", list)).await?;
    Ok(())
}

/// Check if a mentioned user is AFK and notify. Call from event handler on message.
pub async fn check_afk_mention(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    state: &AppState,
) {
    if msg.mentions.is_empty() { return; }
    let Some(guild_id) = msg.guild_id else { return };
    for user in &msg.mentions {
        if user.bot { continue; }
        if let Ok(Some(afk)) = state.db.get_afk(&user.id.to_string(), &guild_id.to_string()).await {
            let _ = msg.channel_id.send_message(&ctx.http,
                serenity::CreateMessage::new().content(
                    format!("{} is AFK: {}", user.name, afk.message)
                ).reference_message((msg.channel_id, msg.id))
            ).await;
        }
    }
}

/// Remove AFK when a user sends a message. Call from event handler.
pub async fn check_afk_return(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    state: &AppState,
) {
    if msg.author.bot { return; }
    let Some(guild_id) = msg.guild_id else { return };
    let uid = msg.author.id.to_string();
    if let Ok(Some(_)) = state.db.get_afk(&uid, &guild_id.to_string()).await {
        state.db.remove_afk(&uid, &guild_id.to_string()).await.unwrap_or_default();
        let _ = msg.channel_id.send_message(&ctx.http,
            serenity::CreateMessage::new().content(format!("Welcome back, {}! I removed your AFK.", msg.author.name))
        ).await;
    }
}
