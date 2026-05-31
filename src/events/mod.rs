use crate::types::{AppState, Error, GuildInfo};
use poise::serenity_prelude as serenity;

pub async fn handle_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, AppState, Error>,
    state: &AppState,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            println!("Logged in as {}", data_about_bot.user.name);
            let mut bot_state = state.bot_state.write().await;
            bot_state.started_at = Some(std::time::SystemTime::now());
            for guild_id in ctx.cache.guilds() {
                let id_str = guild_id.to_string();
                bot_state.bot_guilds.insert(id_str.clone());
                if let Some(guild) = ctx.cache.guild(guild_id) {
                    bot_state.guild_info.insert(id_str, GuildInfo {
                        name: guild.name.clone(),
                        owner_id: guild.owner_id.to_string(),
                        icon: guild.icon.clone().map(|i| i.to_string()),
                    });
                }
            }
        }
        serenity::FullEvent::GuildCreate { guild, .. } => {
            let mut bot_state = state.bot_state.write().await;
            let id_str = guild.id.to_string();
            bot_state.bot_guilds.insert(id_str.clone());
            bot_state.guild_info.insert(id_str, GuildInfo {
                name: guild.name.clone(),
                owner_id: guild.owner_id.to_string(),
                icon: guild.icon.clone().map(|i| i.to_string()),
            });
        }
        serenity::FullEvent::GuildDelete { incomplete, .. } => {
            let mut bot_state = state.bot_state.write().await;
            let id_str = incomplete.id.to_string();
            bot_state.bot_guilds.remove(&id_str);
            bot_state.guild_info.remove(&id_str);
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.author.bot { return Ok(()); }

            // println!("Message from {}: {}", new_message.author.name, new_message.content);

            if let Some(guild_id) = new_message.guild_id {
                let guild_id_str = guild_id.to_string();

                if let Ok(Some(config)) = state.db.get_guild_config(&guild_id_str).await {
                    if config.modules.auto_mod && config.auto_mod.enabled {
                        let auto_mod = crate::modules::auto_mod::AutoMod::new(state.spam_tracker.clone());
                        auto_mod.run(ctx, new_message, &config.auto_mod, state).await;
                    }

                    if config.modules.moderation {
                        let moderation = crate::modules::moderation::ModerationModule::new();
                        let _ = moderation.handle_message(ctx, new_message, state).await;
                    }

                    if config.modules.custom_commands {
                        let prefix = &state.config.read().await.prefix;
                        if !prefix.is_empty() {
                            let cc = crate::modules::custom_commands::CustomCommands::new();
                            cc.run(ctx, new_message, state, prefix).await;
                        }
                    }

                    if config.modules.xp {
                        crate::modules::xp::handle_message_xp(ctx, new_message, state).await;
                    }

                    // AFK checks
                    crate::modules::afk::check_afk_return(ctx, new_message, state).await;
                    crate::modules::afk::check_afk_mention(ctx, new_message, state).await;
                }
            }
        }
        serenity::FullEvent::MessageDelete { channel_id, deleted_message_id, guild_id } => {
            if let Some(guild_id) = guild_id {
                if log_enabled(state, guild_id).await {
                    let logging = crate::modules::logging::Logging::new();
                    logging.log_message_delete(ctx, *guild_id, *channel_id, *deleted_message_id).await;
                }
            }
        }
        serenity::FullEvent::MessageUpdate { old_if_available, new: _, event } => {
            if let Some(guild_id) = event.guild_id {
                if log_enabled(state, &guild_id).await {
                    let old_content = old_if_available.as_ref().map(|m| m.content.clone());
                    let new_content = event.content.clone();
                    let author = event.author.as_ref().map(|u| u.name.clone());
                    let logging = crate::modules::logging::Logging::new();
                    logging.log_message_edit(ctx, guild_id, event.channel_id, old_content, new_content, author).await;
                }
            }
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            let guild_id = new_member.guild_id;
            println!("New member joined: {} (guild {})", new_member.user.name, guild_id);

            if let Ok(Some(config)) = state.db.get_guild_config(&guild_id.to_string()).await {
                if config.modules.welcome {
                    if log_enabled(state, &guild_id).await {
                        let logging = crate::modules::logging::Logging::new();
                        logging.log_member_join(ctx, guild_id, new_member).await;
                    }
                    send_welcome(ctx, &guild_id, new_member, &config.welcome).await;
                }
            }
        }
        serenity::FullEvent::GuildMemberRemoval { guild_id, user, member_data_if_available } => {
            println!("Member left: {} (guild {})", user.name, guild_id);

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
        serenity::FullEvent::GuildMemberUpdate { old_if_available: _, new: _, event } => {
            if log_enabled(state, &event.guild_id).await {
                let logging = crate::modules::logging::Logging::new();
                logging.log_member_update(ctx, event.guild_id, event).await;
            }
        }
        serenity::FullEvent::ChannelCreate { channel } => {
            let guild_id = channel.guild_id;
            if log_enabled(state, &guild_id).await {
                let logging = crate::modules::logging::Logging::new();
                logging.log_channel_create(ctx, guild_id, channel).await;
            }
        }
        serenity::FullEvent::ChannelDelete { channel, .. } => {
            let guild_id = channel.guild_id;
            if log_enabled(state, &guild_id).await {
                let logging = crate::modules::logging::Logging::new();
                logging.log_channel_delete(ctx, guild_id, channel).await;
            }
        }
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            handle_reaction(ctx, add_reaction, state, true).await;
            crate::modules::giveaway::handle_giveaway_reaction(ctx, add_reaction, true, state).await;
            crate::modules::tickets::handle_ticket_reaction(ctx, add_reaction, state).await;
        }
        serenity::FullEvent::ReactionRemove { removed_reaction } => {
            handle_reaction(ctx, removed_reaction, state, false).await;
            crate::modules::giveaway::handle_giveaway_reaction(ctx, removed_reaction, false, state).await;
        }
        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            if let Some(guild_id) = new.guild_id {
                if log_enabled(state, &guild_id).await {
                    let logging = crate::modules::logging::Logging::new();
                    logging.log_voice_state(ctx, guild_id, old.as_ref(), new).await;
                }
            }
        }
        serenity::FullEvent::InteractionCreate { interaction } => {
            if let serenity::Interaction::Component(component) = interaction {
                if component.data.custom_id.starts_with("br_") {
                    crate::commands::fun::handle_banroulette_component(
                        ctx,
                        state,
                        &component.data.custom_id,
                        component,
                    )
                    .await;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_reaction(ctx: &serenity::Context, reaction: &serenity::Reaction, state: &AppState, adding: bool) {
    let Some(guild_id) = reaction.guild_id else { return };
    let Ok(Some(config)) = state.db.get_guild_config(&guild_id.to_string()).await else { return };
    if !config.modules.reaction_roles { return; }

    let emoji_str = reaction.emoji.to_string();
    let msg_id = reaction.message_id.to_string();
    let Ok(roles) = state.db.list_reaction_roles(&guild_id.to_string()).await else { return };

    let Some(rr) = roles.iter().find(|r| r.message_id == msg_id && r.emoji == emoji_str) else { return };

    let user_id = match reaction.user_id {
        Some(uid) => uid,
        None => return,
    };

    let role_id: serenity::RoleId = match rr.role_id.parse() {
        Ok(id) => serenity::RoleId::new(id),
        Err(_) => return,
    };

    if adding {
        if let Ok(member) = guild_id.member(&ctx.http, user_id).await {
            let _ = member.add_role(&ctx.http, role_id).await;
        }
    } else {
        if let Ok(member) = guild_id.member(&ctx.http, user_id).await {
            let _ = member.remove_role(&ctx.http, role_id).await;
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
