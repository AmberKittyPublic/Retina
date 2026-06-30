use crate::database::{self, XpConfig};
use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;
use poise::Command;

pub fn commands() -> Vec<Command<AppState, Error>> {
    vec![rank(), leaderboard(), xp()]
}

#[poise::command(slash_command)]
async fn rank(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to check rank of"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let gid = guild_id.to_string();
    let target = user.as_ref().unwrap_or_else(|| ctx.author());
    let uid = target.id.to_string();

    let data = ctx.data().db.get_xp_data(&gid, &uid).await?.unwrap_or_else(|| {
        // Minimal default
        crate::database::XpData {
            id: 0, guild_id: gid.clone(), user_id: uid.clone(),
            xp: 0, level: 1, last_xp_time: None,
        }
    });

    let level = database::xp_level(data.xp);
    let current_min = database::xp_for_level(level);
    let next_min = database::xp_for_level(level + 1);
    let progress = database::xp_progress(data.xp);
    let _needed = database::xp_to_next_level(data.xp);
    let bar_len = 20;
    let filled = (progress * bar_len as f64).round() as usize;
    let bar: String = (0..bar_len).map(|i| if i < filled { "█" } else { "░" }).collect();

    let embed = serenity::CreateEmbed::new()
        .title(format!("{} — Level {}", target.name, level))
        .description(format!(
            "**XP:** {} ({} / {} to next level)\n`{}`",
            data.xp, data.xp - current_min, next_min - current_min, bar
        ))
        .color(serenity::Colour::GOLD);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn leaderboard(
    ctx: poise::Context<'_, AppState, Error>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let gid = guild_id.to_string();

    let top = ctx.data().db.get_xp_leaderboard(&gid, 10).await?;
    if top.is_empty() {
        ctx.say("No XP data yet. Start chatting to earn XP!").await?;
        return Ok(());
    }

    let mut desc = String::new();
    for (i, entry) in top.iter().enumerate() {
        let name = guild_id.member(ctx, serenity::UserId::new(entry.user_id.parse().unwrap_or(0))).await
            .map(|m| m.user.name).unwrap_or_else(|_| entry.user_id.clone());
        desc.push_str(&format!("**{}.** {} — Level {} ({} XP)\n", i + 1, name, entry.level, entry.xp));
    }

    let embed = serenity::CreateEmbed::new()
        .title("🏆 XP Leaderboard")
        .description(desc)
        .color(serenity::Colour::GOLD);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, subcommands("set", "add", "role_add", "role_remove", "role_list"))]
async fn xp(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    ctx.say("Subcommands: `set`, `add`, `role_add`, `role_remove`, `role_list`").await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn set(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User"] user: serenity::User,
    #[description = "XP amount"] amount: i64,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let gid = guild_id.to_string();
    let uid = user.id.to_string();
    let level = database::xp_level(amount.max(0));
    let now = chrono::Utc::now().to_rfc3339();
    ctx.data().db.upsert_xp_data(&gid, &uid, amount.max(0), level, &now).await?;
    ctx.say(format!("Set {}'s XP to {} (Level {}).", user.name, amount.max(0), level)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn add(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User"] user: serenity::User,
    #[description = "XP to add"] amount: i64,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let gid = guild_id.to_string();
    let uid = user.id.to_string();

    let xp_data = ctx.data().db.get_xp_data(&gid, &uid).await?;
    let (new_xp, old_level) = if let Some(ref d) = xp_data {
        (d.xp + amount, d.level)
    } else {
        (amount.max(0), 1)
    };

    let new_xp = new_xp.max(0);
    let new_level = database::xp_level(new_xp);
    let now = chrono::Utc::now().to_rfc3339();
    ctx.data().db.upsert_xp_data(&gid, &uid, new_xp, new_level, &now).await?;

    let mut msg = format!("Added {} XP to {}. Now at {} XP (Level {}).", amount, user.name, new_xp, new_level);
    if new_level > old_level {
        msg.push_str(&format!(" 🎉 Level up! They reached **Level {}**!", new_level));
        check_role_rewards(ctx, &gid, &uid, new_level).await;
    }

    ctx.say(msg).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn role_add(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Level required"] level: i64,
    #[description = "Role to assign"] role: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    ctx.data().db.add_xp_reward(&guild_id.to_string(), level, &role.id.to_string()).await?;
    ctx.say(format!("Role <@&{}> will be awarded at Level {}.", role.id, level)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn role_remove(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Level to remove reward from"] level: i64,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    ctx.data().db.remove_xp_reward(&guild_id.to_string(), level).await?;
    ctx.say(format!("Removed role reward for Level {}.", level)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn role_list(
    ctx: poise::Context<'_, AppState, Error>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let rewards = ctx.data().db.get_xp_rewards(&guild_id.to_string()).await?;
    if rewards.is_empty() {
        ctx.say("No role rewards configured.").await?;
        return Ok(());
    }

    let desc: String = rewards.iter().map(|r| {
        format!("Level {} → <@&{}>", r.level, r.role_id)
    }).collect::<Vec<_>>().join("\n");

    let embed = serenity::CreateEmbed::new()
        .title("🎖️ XP Role Rewards")
        .description(desc)
        .color(serenity::Colour::GOLD);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

pub async fn handle_message_xp(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    state: &AppState,
) {
    if msg.author.bot { return; }
    let Some(guild_id) = msg.guild_id else { return };
    let gid = guild_id.to_string();
    let uid = msg.author.id.to_string();

    let xp_config = match state.db.get_xp_config(&gid).await {
        Ok(Some(c)) => c,
        _ => {
            let default_cfg = XpConfig {
                guild_id: gid.clone(),
                xp_per_message: 20,
                cooldown_seconds: 60,
                min_chars: 1,
                voice_xp_enabled: false,
                voice_xp_interval_minutes: 5,
            };
            let _ = state.db.set_xp_config(&default_cfg).await;
            XpConfig {
                guild_id: gid.clone(),
                ..XpConfig::default()
            }
        },
    };

    if (msg.content.len() as i64) < xp_config.min_chars { return; }

    // Cooldown check
    let mut add_xp = true;
    if let Ok(Some(existing)) = state.db.get_xp_data(&gid, &uid).await {
        if let Some(ref last_time) = existing.last_xp_time {
            if let Ok(last) = chrono::DateTime::parse_from_rfc3339(last_time) {
                let elapsed = (chrono::Utc::now() - last.with_timezone(&chrono::Utc)).num_seconds();
                if elapsed < xp_config.cooldown_seconds {
                    println!("Message from {}: \"{}\" XP: (0)", msg.author.name, msg.content);
                    add_xp = false;
                }
            }
        }
    }

    if !add_xp { return; }

    let current = state.db.get_xp_data(&gid, &uid).await.unwrap_or(None);
    let (old_xp, old_level) = if let Some(ref d) = current {
        (d.xp, d.level)
    } else {
        (0, 1)
    };

    let variance: i64 = ((msg.id.get() % 11) as i64) - 5;
    let gain = (xp_config.xp_per_message + variance).max(1);
    let new_xp = old_xp + gain;
    let new_level = database::xp_level(new_xp);
    let now = chrono::Utc::now().to_rfc3339();

    let _ = state.db.upsert_xp_data(&gid, &uid, new_xp, new_level, &now).await;
    println!("Message from {}: \"{}\" XP: ({})", msg.author.name, msg.content, gain);


    if new_level > old_level {
        let _ = msg.channel_id.send_message(ctx, serenity::CreateMessage::new().content(
            format!("🎉 **{}** leveled up to **Level {}**!", msg.author.name, new_level),
        )).await;
        check_role_rewards_raw(ctx, state, &gid, &uid, new_level).await;
    }
}

async fn check_role_rewards_raw(
    ctx: &serenity::Context,
    state: &AppState,
    gid: &str,
    uid: &str,
    level: i64,
) {
    let Ok(rewards) = state.db.get_xp_rewards(gid).await else { return };
    for reward in rewards {
        if reward.level <= level {
            let guild_id: serenity::GuildId = match gid.parse() {
                Ok(id) => id,
                _ => continue,
            };
            let user_id: serenity::UserId = match uid.parse() {
                Ok(id) => id,
                _ => continue,
            };
            let role_id: serenity::RoleId = match reward.role_id.parse() {
                Ok(id) => serenity::RoleId::new(id),
                _ => continue,
            };
            if let Ok(member) = guild_id.member(ctx, user_id).await {
                let _ = member.add_role(ctx, role_id).await;
            }
        }
    }
}

async fn check_role_rewards(
    ctx: poise::Context<'_, AppState, Error>,
    gid: &str,
    uid: &str,
    level: i64,
) {
    let Ok(rewards) = ctx.data().db.get_xp_rewards(gid).await else { return };
    let Some(guild_id) = ctx.guild_id() else { return };
    for reward in rewards {
        if reward.level <= level {
            let role_id: serenity::RoleId = match reward.role_id.parse() {
                Ok(id) => serenity::RoleId::new(id),
                _ => continue,
            };
            let user_id: serenity::UserId = match uid.parse() {
                Ok(id) => id,
                _ => continue,
            };
            if let Ok(member) = guild_id.member(ctx, user_id).await {
                let _ = member.add_role(ctx, role_id).await;
            }
        }
    }
}
