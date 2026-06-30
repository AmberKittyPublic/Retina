use poise::serenity_prelude as serenity;
use crate::config::{AutoModConfig, AutoModRule};
use crate::types::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

pub struct AutoMod {
    spam_tracker: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

impl AutoMod {
    pub fn new(spam_tracker: Arc<RwLock<HashMap<String, Vec<Instant>>>>) -> Self {
        AutoMod { spam_tracker }
    }

    pub async fn run(
        &self,
        ctx: &serenity::Context,
        msg: &serenity::Message,
        config: &AutoModConfig,
        state: &AppState,
    ) {
        if !config.enabled || msg.author.bot {
            return;
        }

        let channel_id = msg.channel_id.to_string();

        if !config.channel_whitelist.is_empty() && !config.channel_whitelist.contains(&channel_id) {
            return;
        }
        if config.channel_blacklist.contains(&channel_id) {
            return;
        }

        if let Some(guild_id) = msg.guild_id {
            if let Ok(member) = guild_id.member(ctx, msg.author.id).await {
                let user_roles: Vec<String> = member.roles.iter().map(|r| r.to_string()).collect();

                if !config.role_whitelist.is_empty()
                    && !user_roles.iter().any(|r| config.role_whitelist.contains(r))
                {
                    return;
                }
                if user_roles.iter().any(|r| config.role_blacklist.contains(r)) {
                    return;
                }
            }
        }

        for rule in &config.rules {
            if !rule.enabled {
                continue;
            }

            if let Some(reason) = self.check_rule(rule, msg).await {
                println!("Auto-mod triggered [{}]: {}", rule.rule_type, reason);
                if let Err(e) = self.execute_action(ctx, msg, rule, &reason, state).await {
                    eprintln!("Auto-mod action failed: {}", e);
                }
                return;
            }
        }
    }

    async fn check_rule(&self, rule: &AutoModRule, msg: &serenity::Message) -> Option<String> {
        match rule.rule_type.as_str() {
            "spam" => self.check_spam(rule, msg).await,
            "caps" => check_caps(rule, msg),
            "links" => check_links(msg),
            "mentions" => check_mentions(rule, msg),
            "emotes" => check_emotes(rule, msg),
            "banned_words" => check_banned_words(rule, msg),
            "max_length" => check_max_length(rule, msg),
            _ => None,
        }
    }

    async fn check_spam(&self, rule: &AutoModRule, msg: &serenity::Message) -> Option<String> {
        let max = rule.max_messages.unwrap_or(5) as usize;
        let window_secs = rule.window_seconds.unwrap_or(5);

        let key = format!("{}:{}", msg.guild_id?.to_string(), msg.author.id.to_string());
        let now = Instant::now();
        let cutoff = now - std::time::Duration::from_secs(window_secs as u64);

        let mut tracker = self.spam_tracker.write().await;
        let timestamps = tracker.entry(key).or_default();

        timestamps.retain(|t| *t > cutoff);
        timestamps.push(now);

        if timestamps.len() > max {
            Some(format!("Spam detected: {} messages in {}s", timestamps.len(), window_secs))
        } else {
            None
        }
    }

    async fn execute_action(
        &self,
        ctx: &serenity::Context,
        msg: &serenity::Message,
        rule: &AutoModRule,
        reason: &str,
        state: &AppState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let guild_id = match msg.guild_id {
            Some(id) => id,
            None => return Ok(()),
        };

        match rule.action.as_str() {
            "delete" => {
                msg.delete(ctx).await?;
            }
            "warn" => {
                let bot_id = ctx.cache.current_user().id;
                state.db.add_warning(
                    &guild_id.to_string(),
                    &msg.author.id.to_string(),
                    &bot_id.to_string(),
                    reason,
                ).await?;

                let _ = msg.author.dm(ctx, serenity::CreateMessage::new().content(
                    format!("⚠️ **Auto-mod warning** in {}: {}", guild_id.name(ctx).unwrap_or_default(), reason),
                )).await;
            }
            "timeout" => {
                if let Some(minutes) = rule.action_duration_minutes {
                    let until = chrono::Utc::now()
                        + chrono::Duration::minutes(minutes as i64);
                    if let Ok(mut member) = guild_id.member(ctx, msg.author.id).await {
                        member.disable_communication_until_datetime(ctx, until.into()).await?;
                    }
                }
            }
            "kick" => {
                guild_id.kick_with_reason(ctx, msg.author.id, reason).await?;
            }
            "ban" => {
                guild_id.ban_with_reason(ctx, msg.author.id, 0, reason).await?;
            }
            _ => {}
        }

        Ok(())
    }
}

fn check_caps(rule: &AutoModRule, msg: &serenity::Message) -> Option<String> {
    let threshold = rule.caps_percent.unwrap_or(70) as f32;
    let content = &msg.content;

    let letters: usize = content.chars().filter(|c| c.is_alphabetic()).count();
    if letters == 0 {
        return None;
    }

    let caps: usize = content.chars().filter(|c| c.is_uppercase()).count();
    let ratio = caps as f32 / letters as f32 * 100.0;

    if ratio >= threshold && content.len() > 10 {
        Some(format!("Excessive caps: {:.0}% uppercase (limit: {:.0}%)", ratio, threshold))
    } else {
        None
    }
}

fn check_links(msg: &serenity::Message) -> Option<String> {
    if msg.content.contains("http://") || msg.content.contains("https://") {
        Some("Message contains a link".to_string())
    } else {
        None
    }
}

fn check_mentions(rule: &AutoModRule, msg: &serenity::Message) -> Option<String> {
    let max = rule.max_mentions.unwrap_or(5) as usize;

    if msg.mentions.len() > max {
        Some(format!("Mass mention: {} mentions (limit: {})", msg.mentions.len(), max))
    } else {
        None
    }
}

fn check_emotes(rule: &AutoModRule, msg: &serenity::Message) -> Option<String> {
    let max = rule.max_emotes.unwrap_or(5) as usize;

    let count = count_custom_emotes(&msg.content);

    if count > max {
        Some(format!("Emote spam: {} custom emotes (limit: {})", count, max))
    } else {
        None
    }
}

fn check_banned_words(rule: &AutoModRule, msg: &serenity::Message) -> Option<String> {
    if rule.banned_words.is_empty() {
        return None;
    }

    let lower = msg.content.to_lowercase();
    for word in &rule.banned_words {
        if lower.contains(&word.to_lowercase()) {
            return Some(format!("Message contains banned word: {}", word));
        }
    }
    None
}

fn check_max_length(rule: &AutoModRule, msg: &serenity::Message) -> Option<String> {
    let max = rule.max_length.unwrap_or(2000);
    if msg.content.len() > max {
        Some(format!("Message exceeds max length ({} > {})", msg.content.len(), max))
    } else {
        None
    }
}

fn count_custom_emotes(content: &str) -> usize {
    content.matches("<:").count() + content.matches("<a:").count()
}
