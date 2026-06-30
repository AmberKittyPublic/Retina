use crate::types::AppState;
use mlua::Lua;
use poise::serenity_prelude as serenity;
use std::sync::{Arc, Mutex};

enum Action {
    Send(u64, String),
    Reply(u64, u64, String),
    Ban(u64, u64),
    Kick(u64, u64),
    AddRole(u64, u64, u64),
    RemoveRole(u64, u64, u64),
    Timeout(u64, u64, i64),
    Warn(u64, u64, String),
    Embed(u64, String, String, u32),
    React(u64, u64, String),
    DeleteMessage(u64, u64),
    EditMessage(u64, u64, String),
}

pub struct CustomCommands;

impl CustomCommands {
    pub fn new() -> Self {
        CustomCommands
    }

    pub async fn run(
        &self,
        ctx: &serenity::Context,
        msg: &serenity::Message,
        state: &AppState,
        prefix: &str,
    ) {
        let content = msg.content.trim();
        if !content.starts_with(prefix) {
            return;
        }

        let rest = content[prefix.len()..].trim();
        let cmd_name = rest.split_whitespace().next().unwrap_or("");

        if cmd_name.is_empty() {
            return;
        }

        let Some(guild_id) = msg.guild_id else {
            return;
        };

        let cmd = match state.db.get_custom_command(&guild_id.to_string(), cmd_name).await {
            Ok(Some(c)) if c.enabled => c,
            _ => return,
        };

        let args: Vec<String> = rest.split_whitespace().skip(1).map(|s| s.to_string()).collect();

        let script = cmd.script.clone();
        let channel_id = msg.channel_id;
        let msg_id = msg.id;
        let author_id = msg.author.id;
        let author_name = msg.author.name.clone();
        let author_mention = format!("<@{}>", author_id);
        let msg_content = msg.content.clone();
        let guild_id_num = guild_id.get();
        let guild_name = ctx.cache.guild(guild_id).map(|g| g.name.clone()).unwrap_or_default();
        let channel_name = ctx.cache.guild(guild_id)
            .map(|g| g.channels.get(&channel_id).map(|c| c.name.clone()))
            .flatten()
            .unwrap_or_default();

        let author_role_ids: Vec<u64> = if let Ok(member) = guild_id.member(ctx, author_id).await {
            member.roles.iter().map(|r| r.get()).collect()
        } else {
            vec![]
        };

        let ctx = Arc::new(ctx.clone());
        let actions: Arc<Mutex<Vec<Action>>> = Arc::new(Mutex::new(Vec::new()));

        let result = tokio::task::spawn_blocking({
            let actions = Arc::clone(&actions);
            let author_role_ids = author_role_ids.clone();
            let script = script.clone();
            let args = args.clone();
            let author_name = author_name.clone();
            let author_mention = author_mention.clone();
            let msg_content = msg_content.clone();
            let guild_name = guild_name.clone();
            let channel_name = channel_name.clone();
            move || {
                Self::execute_lua(
                    &script, channel_id, msg_id, author_id, &author_name, &author_mention,
                    guild_id_num, &guild_name, &channel_name, &msg_content, &args,
                    &author_role_ids, &actions,
                )
            }
        }).await;

        match result {
            Ok(Ok(reply)) => {
                let pending = actions.lock().unwrap().drain(..).collect::<Vec<_>>();
                for action in pending {
                    Self::execute_action(&ctx, &state, action).await;
                }
                if !reply.is_empty() {
                    let _ = channel_id.send_message(
                        &ctx.http,
                        serenity::CreateMessage::new()
                            .content(reply)
                            .reference_message((channel_id, msg_id)),
                    ).await;
                }
            }
            Ok(Err(e)) => {
                eprintln!("Custom command error: {}", e);
                let _ = channel_id.send_message(
                    &ctx.http,
                    serenity::CreateMessage::new()
                        .content(format!("❌ {}", e))
                        .reference_message((channel_id, msg_id)),
                ).await;
            }
            Err(e) => {
                eprintln!("Custom command join error: {}", e);
            }
        }
    }

    fn execute_lua(
        script: &str,
        channel_id: serenity::ChannelId,
        msg_id: serenity::MessageId,
        author_id: serenity::UserId,
        author_name: &str,
        author_mention: &str,
        guild_id_num: u64,
        guild_name: &str,
        channel_name: &str,
        msg_content: &str,
        args: &[String],
        author_role_ids: &[u64],
        actions: &Arc<Mutex<Vec<Action>>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let lua = Lua::new();

        let globals = lua.globals();

        let safe_globals = ["assert", "error", "ipairs", "next", "pairs", "pcall",
            "print", "select", "tonumber", "tostring", "type", "unpack", "xpcall",
            "_VERSION", "coroutine", "math", "string", "table", "utf8"];

        let mut keys_to_remove = Vec::new();
        for pair in globals.pairs::<String, mlua::Value>() {
            let (key, _) = pair?;
            if !safe_globals.contains(&key.as_str()) {
                keys_to_remove.push(key);
            }
        }
        for key in keys_to_remove {
            globals.raw_remove(key)?;
        }

        let gid = guild_id_num;
        let cid = channel_id.get();
        let mid = msg_id.get();

        let api = lua.create_table()?;

        let actions_clone = Arc::clone(actions);
        let send_fn = lua.create_function(move |_, (ch_id, content): (u64, String)| {
            actions_clone.lock().unwrap().push(Action::Send(ch_id, content));
            Ok(())
        })?;
        api.set("send", send_fn)?;

        let actions_clone = Arc::clone(actions);
        let reply_fn = lua.create_function(move |_, content: String| {
            actions_clone.lock().unwrap().push(Action::Reply(cid, mid, content));
            Ok(())
        })?;
        api.set("reply", reply_fn)?;

        let actions_clone = Arc::clone(actions);
        let ban_fn = lua.create_function(move |_, (uid, _reason): (u64, String)| {
            actions_clone.lock().unwrap().push(Action::Ban(gid, uid));
            Ok(())
        })?;
        api.set("ban", ban_fn)?;

        let actions_clone = Arc::clone(actions);
        let kick_fn = lua.create_function(move |_, uid: u64| {
            actions_clone.lock().unwrap().push(Action::Kick(gid, uid));
            Ok(())
        })?;
        api.set("kick", kick_fn)?;

        let actions_clone = Arc::clone(actions);
        let add_role_fn = lua.create_function(move |_, (uid, rid): (u64, u64)| {
            actions_clone.lock().unwrap().push(Action::AddRole(gid, uid, rid));
            Ok(())
        })?;
        api.set("add_role", add_role_fn)?;

        let actions_clone = Arc::clone(actions);
        let remove_role_fn = lua.create_function(move |_, (uid, rid): (u64, u64)| {
            actions_clone.lock().unwrap().push(Action::RemoveRole(gid, uid, rid));
            Ok(())
        })?;
        api.set("remove_role", remove_role_fn)?;

        let actions_clone = Arc::clone(actions);
        let timeout_fn = lua.create_function(move |_, (uid, minutes): (u64, u64)| {
            actions_clone.lock().unwrap().push(Action::Timeout(gid, uid, minutes.min(40320) as i64));
            Ok(())
        })?;
        api.set("timeout", timeout_fn)?;

        let actions_clone = Arc::clone(actions);
        let warn_fn = lua.create_function(move |_, (uid, reason): (u64, String)| {
            actions_clone.lock().unwrap().push(Action::Warn(gid, uid, reason));
            Ok(())
        })?;
        api.set("warn", warn_fn)?;

        let actions_clone = Arc::clone(actions);
        let embed_fn = lua.create_function(move |_, (title, description, color): (String, String, u32)| {
            actions_clone.lock().unwrap().push(Action::Embed(cid, title, description, color));
            Ok(())
        })?;
        api.set("embed", embed_fn)?;

        let actions_clone = Arc::clone(actions);
        let embed_to_fn = lua.create_function(move |_, (ch_id, title, description, color): (u64, String, String, u32)| {
            actions_clone.lock().unwrap().push(Action::Embed(ch_id, title, description, color));
            Ok(())
        })?;
        api.set("embed_to", embed_to_fn)?;

        let actions_clone = Arc::clone(actions);
        let react_fn = lua.create_function(move |_, emoji: String| {
            actions_clone.lock().unwrap().push(Action::React(cid, mid, emoji));
            Ok(())
        })?;
        api.set("react", react_fn)?;

        let actions_clone = Arc::clone(actions);
        let add_reaction_fn = lua.create_function(move |_, (ch_id, m_id, emoji): (u64, u64, String)| {
            actions_clone.lock().unwrap().push(Action::React(ch_id, m_id, emoji));
            Ok(())
        })?;
        api.set("add_reaction", add_reaction_fn)?;

        let actions_clone = Arc::clone(actions);
        let delete_fn = lua.create_function(move |_, ()| {
            actions_clone.lock().unwrap().push(Action::DeleteMessage(cid, mid));
            Ok(())
        })?;
        api.set("delete", delete_fn)?;

        let actions_clone = Arc::clone(actions);
        let delete_message_fn = lua.create_function(move |_, (ch_id, m_id): (u64, u64)| {
            actions_clone.lock().unwrap().push(Action::DeleteMessage(ch_id, m_id));
            Ok(())
        })?;
        api.set("delete_message", delete_message_fn)?;

        let actions_clone = Arc::clone(actions);
        let edit_fn = lua.create_function(move |_, content: String| {
            actions_clone.lock().unwrap().push(Action::EditMessage(cid, mid, content));
            Ok(())
        })?;
        api.set("edit", edit_fn)?;

        let author_roles = author_role_ids.to_vec();
        let has_role_fn = lua.create_function(move |_, role_id: u64| {
            Ok(author_roles.contains(&role_id))
        })?;
        api.set("has_role", has_role_fn)?;

        globals.set("api", api)?;

        let msg_table = lua.create_table()?;
        msg_table.set("author_id", author_id.get())?;
        msg_table.set("author_name", author_name)?;
        msg_table.set("author_mention", author_mention)?;
        msg_table.set("channel_id", cid)?;
        msg_table.set("channel_name", channel_name)?;
        msg_table.set("guild_id", gid)?;
        msg_table.set("guild_name", guild_name)?;
        msg_table.set("content", msg_content)?;
        globals.set("message", msg_table)?;

        let args_table = lua.create_table()?;
        for (i, arg) in args.iter().enumerate() {
            args_table.set(i + 1, arg.clone())?;
        }
        globals.set("args", args_table)?;

        let result: String = lua.load(script).eval()?;
        Ok(result)
    }


    async fn execute_action(ctx: &serenity::Context, state: &AppState, action: Action) {
        match action {
            Action::Send(ch_id, content) => {
                let ch = serenity::ChannelId::new(ch_id);
                let _ = ch.send_message(&ctx.http, serenity::CreateMessage::new().content(content)).await;
            }
            Action::Reply(ch_id, msg_id, content) => {
                let ch = serenity::ChannelId::new(ch_id);
                let _ = ch.send_message(
                    &ctx.http,
                    serenity::CreateMessage::new()
                        .content(content)
                        .reference_message((serenity::ChannelId::new(ch_id), serenity::MessageId::new(msg_id))),
                ).await;
            }
            Action::Ban(guild_id, uid) => {
                let g = serenity::GuildId::new(guild_id);
                let _ = g.ban(&ctx.http, serenity::UserId::new(uid), 0).await;
            }
            Action::Kick(guild_id, uid) => {
                let g = serenity::GuildId::new(guild_id);
                let _ = g.kick(&ctx.http, serenity::UserId::new(uid)).await;
            }
            Action::AddRole(guild_id, uid, rid) => {
                let g = serenity::GuildId::new(guild_id);
                if let Ok(member) = g.member(&ctx.http, serenity::UserId::new(uid)).await {
                    let _ = member.add_role(&ctx.http, serenity::RoleId::new(rid)).await;
                }
            }
            Action::RemoveRole(guild_id, uid, rid) => {
                let g = serenity::GuildId::new(guild_id);
                if let Ok(member) = g.member(&ctx.http, serenity::UserId::new(uid)).await {
                    let _ = member.remove_role(&ctx.http, serenity::RoleId::new(rid)).await;
                }
            }
            Action::Timeout(guild_id, uid, minutes) => {
                let g = serenity::GuildId::new(guild_id);
                if let Ok(mut member) = g.member(&ctx.http, serenity::UserId::new(uid)).await {
                    let until = chrono::Utc::now() + chrono::Duration::minutes(minutes);
                    let _ = member.disable_communication_until_datetime(&ctx.http, until.into()).await;
                }
            }
            Action::Warn(guild_id, uid, reason) => {
                let bot_id = ctx.cache.current_user().id;
                let _ = state.db.add_warning(
                    &guild_id.to_string(),
                    &uid.to_string(),
                    &bot_id.to_string(),
                    &reason,
                ).await;
            }
            Action::Embed(ch_id, title, description, color) => {
                let ch = serenity::ChannelId::new(ch_id);
                let embed = serenity::CreateEmbed::new()
                    .title(title)
                    .description(description)
                    .color(serenity::Colour::new(color));
                let _ = ch.send_message(&ctx.http, serenity::CreateMessage::new().embed(embed)).await;
            }
            Action::React(ch_id, m_id, emoji) => {
                let ch = serenity::ChannelId::new(ch_id);
                let msg = serenity::MessageId::new(m_id);
                let reaction = if emoji.starts_with('<') {
                    serenity::ReactionType::Custom {
                        animated: emoji.starts_with("<a:"),
                        id: emoji.split(':').nth(2).and_then(|s| s.trim_end_matches('>').parse().ok())
                            .unwrap_or(serenity::EmojiId::new(0)),
                        name: Some(emoji.split(':').nth(1).unwrap_or("").to_string()),
                    }
                } else {
                    serenity::ReactionType::Unicode(emoji)
                };
                let _ = ch.create_reaction(&ctx.http, msg, reaction).await;
            }
            Action::DeleteMessage(ch_id, m_id) => {
                let ch = serenity::ChannelId::new(ch_id);
                let _ = ch.delete_message(&ctx.http, serenity::MessageId::new(m_id)).await;
            }
            Action::EditMessage(ch_id, m_id, content) => {
                let ch = serenity::ChannelId::new(ch_id);
                let _ = ch.edit_message(&ctx.http, serenity::MessageId::new(m_id), serenity::EditMessage::new().content(content)).await;
            }
        }
    }
}
