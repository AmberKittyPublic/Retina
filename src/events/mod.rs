use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;

pub async fn handle_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, AppState, Error>,
    state: &AppState,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            crate::modules::general::handle_ready(ctx, data_about_bot, state).await;
        }
        serenity::FullEvent::GuildCreate { guild, .. } => {
            crate::modules::general::handle_guild_create(guild, state).await;
        }
        serenity::FullEvent::GuildDelete { incomplete, .. } => {
            crate::modules::general::handle_guild_delete(&incomplete.id, state).await;
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.author.bot { return Ok(()); }

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

                    crate::modules::afk::check_afk_return(ctx, new_message, state).await;
                    crate::modules::afk::check_afk_mention(ctx, new_message, state).await;
                }
            }
        }
        serenity::FullEvent::MessageDelete { channel_id, deleted_message_id, guild_id } => {
            if let Some(guild_id) = guild_id {
                if let Ok(Some(config)) = state.db.get_guild_config(&guild_id.to_string()).await {
                    if config.modules.logging {
                        let logging = crate::modules::logging::Logging::new();
                        logging.log_message_delete(ctx, *guild_id, *channel_id, *deleted_message_id).await;
                    }
                }
            }
        }
        serenity::FullEvent::MessageUpdate { old_if_available, new: _, event } => {
            if let Some(guild_id) = event.guild_id {
                if let Ok(Some(config)) = state.db.get_guild_config(&guild_id.to_string()).await {
                    if config.modules.logging {
                        let old_content = old_if_available.as_ref().map(|m| m.content.clone());
                        let new_content = event.content.clone();
                        let author = event.author.as_ref().map(|u| u.name.clone());
                        let logging = crate::modules::logging::Logging::new();
                        logging.log_message_edit(ctx, guild_id, event.channel_id, old_content, new_content, author).await;
                    }
                }
            }
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            println!("New member joined: {} (guild {})", new_member.user.name, new_member.guild_id);
            crate::modules::welcome::handle_member_add(ctx, &new_member.guild_id, new_member, state).await;
        }
        serenity::FullEvent::GuildMemberRemoval { guild_id, user, member_data_if_available } => {
            println!("Member left: {} (guild {})", user.name, guild_id);
            crate::modules::welcome::handle_member_remove(ctx, guild_id, user, member_data_if_available.as_ref(), state).await;
        }
        serenity::FullEvent::GuildMemberUpdate { old_if_available: _, new: _, event } => {
            if let Ok(Some(config)) = state.db.get_guild_config(&event.guild_id.to_string()).await {
                if config.modules.logging {
                    let logging = crate::modules::logging::Logging::new();
                    logging.log_member_update(ctx, event.guild_id, event).await;
                }
            }
        }
        serenity::FullEvent::ChannelCreate { channel } => {
            let guild_id = channel.guild_id;
            if let Ok(Some(config)) = state.db.get_guild_config(&guild_id.to_string()).await {
                if config.modules.logging {
                    let logging = crate::modules::logging::Logging::new();
                    logging.log_channel_create(ctx, guild_id, channel).await;
                }
            }
        }
        serenity::FullEvent::ChannelDelete { channel, .. } => {
            let guild_id = channel.guild_id;
            if let Ok(Some(config)) = state.db.get_guild_config(&guild_id.to_string()).await {
                if config.modules.logging {
                    let logging = crate::modules::logging::Logging::new();
                    logging.log_channel_delete(ctx, guild_id, channel).await;
                }
            }
        }
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            crate::modules::reaction_roles::handle_reaction(ctx, add_reaction, state, true).await;
            crate::modules::giveaway::handle_giveaway_reaction(ctx, add_reaction, true, state).await;
            crate::modules::tickets::handle_ticket_reaction(ctx, add_reaction, state).await;
        }
        serenity::FullEvent::ReactionRemove { removed_reaction } => {
            crate::modules::reaction_roles::handle_reaction(ctx, removed_reaction, state, false).await;
            crate::modules::giveaway::handle_giveaway_reaction(ctx, removed_reaction, false, state).await;
        }
        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            if let Some(guild_id) = new.guild_id {
                if let Ok(Some(config)) = state.db.get_guild_config(&guild_id.to_string()).await {
                    if config.modules.logging {
                        let logging = crate::modules::logging::Logging::new();
                        logging.log_voice_state(ctx, guild_id, old.as_ref(), new).await;
                    }
                }
            }
        }
        serenity::FullEvent::InteractionCreate { interaction } => {
            if let serenity::Interaction::Component(component) = interaction {
                if component.data.custom_id.starts_with("br_") {
                    crate::modules::fun::handle_banroulette_component(
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
