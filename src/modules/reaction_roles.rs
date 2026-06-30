use crate::types::AppState;
use poise::serenity_prelude as serenity;

pub async fn handle_reaction(ctx: &serenity::Context, reaction: &serenity::Reaction, state: &AppState, adding: bool) {
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
