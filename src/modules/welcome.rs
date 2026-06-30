use crate::types::AppState;
use poise::serenity_prelude as serenity;

pub async fn handle_member_add(ctx: &serenity::Context, guild_id: &serenity::GuildId, member: &serenity::Member, state: &AppState) {
    if let Ok(Some(config)) = state.db.get_guild_config(&guild_id.to_string()).await {
        if config.modules.welcome {
            if log_enabled(state, guild_id).await {
                let logging = crate::modules::logging::Logging::new();
                logging.log_member_join(ctx, *guild_id, member).await;
            }
            send_welcome(ctx, guild_id, member, &config.welcome).await;
        }
    }
}

pub async fn handle_member_remove(ctx: &serenity::Context, guild_id: &serenity::GuildId, user: &serenity::User, member_data_if_available: Option<&serenity::Member>, state: &AppState) {
    if let Ok(Some(config)) = state.db.get_guild_config(&guild_id.to_string()).await {
        if config.modules.welcome {
            if log_enabled(state, guild_id).await {
                let roles: Vec<serenity::Role> = if let Some(ref m) = member_data_if_available {
                    ctx.cache.guild(*guild_id)
                        .map(|g| {
                            m.roles.iter()
                                .filter_map(|role_id| g.roles.get(role_id))
                                .cloned()
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    vec![]
                };
                let logging = crate::modules::logging::Logging::new();
                logging.log_member_leave(ctx, *guild_id, user, &roles).await;
            }
            send_goodbye(ctx, *guild_id, user, &config.welcome).await;
        }
    }
}

async fn send_welcome(ctx: &serenity::Context, guild_id: &serenity::GuildId, member: &serenity::Member, config: &crate::config::WelcomeConfig) {
    if config.welcome_channel_id.is_empty() {
        return;
    }
    let channel_id: serenity::ChannelId = match config.welcome_channel_id.parse() {
        Ok(id) => id,
        Err(_) => return,
    };
    let guild_name = ctx.cache.guild(*guild_id).map(|g| g.name.clone()).unwrap_or_default();
    let msg = config.welcome_message
        .replace("{user}", &member.user.name)
        .replace("{mention}", &format!("<@{}>", member.user.id))
        .replace("{guild}", &guild_name);
    let _ = channel_id.send_message(&ctx.http, serenity::CreateMessage::new().content(msg)).await;
}

async fn send_goodbye(ctx: &serenity::Context, guild_id: serenity::GuildId, user: &serenity::User, config: &crate::config::WelcomeConfig) {
    if config.goodbye_channel_id.is_empty() {
        return;
    }
    let channel_id: serenity::ChannelId = match config.goodbye_channel_id.parse() {
        Ok(id) => id,
        Err(_) => return,
    };
    let guild_name = ctx.cache.guild(guild_id).map(|g| g.name.clone()).unwrap_or_default();
    let msg = config.goodbye_message
        .replace("{user}", &user.name)
        .replace("{mention}", &format!("<@{}>", user.id))
        .replace("{guild}", &guild_name);
    let _ = channel_id.send_message(&ctx.http, serenity::CreateMessage::new().content(msg)).await;
}

async fn log_enabled(state: &AppState, guild_id: &serenity::GuildId) -> bool {
    if let Ok(Some(config)) = state.db.get_guild_config(&guild_id.to_string()).await {
        config.modules.logging
    } else {
        false
    }
}
