use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;
use poise::Command;
use rand::seq::SliceRandom;

pub fn commands() -> Vec<Command<AppState, Error>> {
    vec![giveaway()]
}

#[poise::command(slash_command, subcommands("create", "reroll", "end"))]
async fn giveaway(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    ctx.say("Subcommands: `create`, `reroll`, `end`").await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn create(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Prize name"] prize: String,
    #[description = "Duration in minutes"] duration_minutes: u32,
    #[description = "Number of winners (default 1)"] winners: Option<u32>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let channel_id = ctx.channel_id();
    let winners_count = winners.unwrap_or(1).max(1) as i64;

    let end_time = chrono::Utc::now() + chrono::Duration::minutes(duration_minutes as i64);

    let embed = serenity::CreateEmbed::new()
        .title("🎉 Giveaway!")
        .description(format!(
            "**Prize:** {prize}\n**Winners:** {count}\n**Ends:** <t:{ts}:R>\n\nReact with 🎉 to enter!",
            prize = prize,
            count = winners_count,
            ts = end_time.timestamp(),
        ))
        .color(serenity::Colour::GOLD);

    let msg = channel_id.send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    msg.react(ctx, serenity::ReactionType::Unicode("🎉".to_string())).await?;

    ctx.data().db.create_giveaway(
        &guild_id.to_string(),
        &channel_id.to_string(),
        &msg.id.to_string(),
        &prize,
        winners_count,
        &end_time.to_rfc3339(),
    ).await?;

    ctx.say(format!("Giveaway created for **{prize}**! Ends <t:{ts}:R>.", ts = end_time.timestamp())).await?;

    schedule_giveaway_end(ctx.data().clone(), msg.id, channel_id, guild_id, end_time);

    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn reroll(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Message ID of the giveaway"] message_id: String,
    #[description = "Number of new winners"] count: Option<u32>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let gid = guild_id.to_string();
    let Some(mut ga) = ctx.data().db.get_giveaway_by_message(&gid, &message_id).await? else {
        return Err("Giveaway not found".into());
    };
    if !ga.ended {
        ctx.data().db.end_giveaway(&gid, &message_id).await?;
        ga.ended = true;
    }

    let entries = ga.entries_vec();
    if entries.is_empty() {
        return Err("No entries to pick from".into());
    }

    let pick_count = (count.unwrap_or(1) as usize).min(entries.len());
    let winners: Vec<&String> = entries.choose_multiple(&mut rand::thread_rng(), pick_count).collect();

    let msg = format!("🎉 **Reroll!** New winner(s) for **{prize}**: {winners}",
        prize = ga.prize, winners = winners.iter().map(|u| format!("<@{u}>")).collect::<Vec<_>>().join(", "));
    ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().content(msg)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn end(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Message ID of the giveaway"] message_id: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let channel_id = ctx.channel_id();
    let mid: serenity::MessageId = message_id.parse().map_err(|_| "Invalid message ID")?;
    finish_giveaway(ctx.data(), &guild_id, &channel_id, &mid).await?;
    ctx.say("Giveaway ended.").await?;
    Ok(())
}

pub async fn finish_giveaway(
    state: &AppState,
    guild_id: &serenity::GuildId,
    channel_id: &serenity::ChannelId,
    message_id: &serenity::MessageId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let gid = guild_id.to_string();
    let mid = message_id.to_string();

    let Some(mut ga) = state.db.get_giveaway_by_message(&gid, &mid).await? else {
        return Ok(());
    };
    if ga.ended {
        return Ok(());
    }

    state.db.end_giveaway(&gid, &mid).await?;
    ga.ended = true;

    let entries = ga.entries_vec();
    let mut winners_text = String::from("No one entered 😢");

    if !entries.is_empty() {
        let pick_count = (ga.winners_count as usize).min(entries.len());
        let winners: Vec<&String> = entries.choose_multiple(&mut rand::thread_rng(), pick_count).collect();
        winners_text = winners.iter().map(|u| format!("<@{u}>")).collect::<Vec<_>>().join(", ");
    }

    let embed = serenity::CreateEmbed::new()
        .title("🎉 Giveaway Ended!")
        .description(format!(
            "**Prize:** {prize}\n**Winners:** {winners}\n**Total Entries:** {total}",
            prize = ga.prize,
            winners = winners_text,
            total = entries.len(),
        ))
        .color(serenity::Colour::DARK_GREEN);

    let http = serenity::Http::new(&state.settings.discord.token);

    let _ = channel_id.edit_message(&http, *message_id, serenity::EditMessage::new().embed(embed)).await;

    let _ = channel_id.send_message(
        &http,
        serenity::CreateMessage::new()
            .content(format!("🎉 Congratulations {winners_text}! You won **{prize}**!", prize = ga.prize))
            .reference_message((*channel_id, *message_id)),
    ).await;

    Ok(())
}

fn schedule_giveaway_end(
    state: AppState,
    message_id: serenity::MessageId,
    channel_id: serenity::ChannelId,
    guild_id: serenity::GuildId,
    end_time: chrono::DateTime<chrono::Utc>,
) {
    tokio::spawn(async move {
        let duration = (end_time - chrono::Utc::now())
            .max(chrono::Duration::zero())
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(0));

        if duration > std::time::Duration::from_secs(0) {
            tokio::time::sleep(duration).await;
        }
        if let Err(e) = finish_giveaway(&state, &guild_id, &channel_id, &message_id).await {
            eprintln!("Giveaway finish error: {}", e);
        }
    });
}

pub async fn handle_giveaway_reaction(
    _ctx: &serenity::Context,
    reaction: &serenity::Reaction,
    adding: bool,
    state: &AppState,
) {
    let Some(guild_id) = reaction.guild_id else { return };
    let mid = reaction.message_id.to_string();
    let gid = guild_id.to_string();

    let mut ga = match state.db.get_giveaway_by_message(&gid, &mid).await {
        Ok(Some(g)) if !g.ended => g,
        _ => return,
    };

    let Some(user_id) = reaction.user_id else { return };
    let uid = user_id.to_string();

    let changed = if adding {
        ga.add_entry(&uid)
    } else {
        ga.remove_entry(&uid)
    };

    if changed {
        let _ = state.db.update_giveaway_entries(&gid, &mid, &ga.entries).await;
    }
}

pub async fn check_expired_giveaways(state: &AppState) {
    let Ok(giveaways) = state.db.list_active_giveaways().await else { return };
    for ga in &giveaways {
        let guild_id = serenity::GuildId::new(ga.guild_id.parse().unwrap_or(0));
        let channel_id = serenity::ChannelId::new(ga.channel_id.parse().unwrap_or(0));
        let message_id = serenity::MessageId::new(ga.message_id.parse().unwrap_or(0));

        if ga.is_expired() {
            if let Err(e) = finish_giveaway(state, &guild_id, &channel_id, &message_id).await {
                eprintln!("Giveaway expired finish error: {}", e);
            }
        } else {
            if let Ok(end) = chrono::DateTime::parse_from_rfc3339(&ga.end_time) {
                schedule_giveaway_end(state.clone(), message_id, channel_id, guild_id, end.into());
            }
        }
    }
}
