use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;
use poise::Command;

pub struct ManagerModule;

impl ManagerModule {
    pub fn new() -> Self { ManagerModule }
}

pub fn commands() -> Vec<Command<AppState, Error>> {
    vec![addmod(), delmod(), listmods(), nick(), addrole(), delrole()]
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn addmod(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Role to add as moderator"] role: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    ctx.data().db.add_moderator_role(&guild_id.to_string(), &role.id.to_string()).await?;
    ctx.say(format!("Added {} as a moderator role.", role.name)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn delmod(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Role to remove as moderator"] role: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    ctx.data().db.remove_moderator_role(&guild_id.to_string(), &role.id.to_string()).await?;
    ctx.say(format!("Removed {} as a moderator role.", role.name)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_GUILD")]
async fn listmods(
    ctx: poise::Context<'_, AppState, Error>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let roles = ctx.data().db.list_moderator_roles(&guild_id.to_string()).await?;
    if roles.is_empty() {
        ctx.say("No moderator roles configured.").await?;
        return Ok(());
    }
    let list: String = roles.iter().map(|r| format!("<@&{}>", r.role_id)).collect::<Vec<_>>().join(", ");
    let embed = serenity::CreateEmbed::new()
        .title("Moderator Roles")
        .description(list)
        .color(serenity::Colour::BLUE);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_NICKNAMES")]
async fn nick(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to change nickname for"] user: serenity::User,
    #[description = "New nickname (empty to reset)"] nickname: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let mut member = guild_id.member(ctx, user.id).await?;
    match nickname {
        Some(n) if !n.is_empty() => {
            member.edit(ctx, serenity::EditMember::new().nickname(&n)).await?;
            ctx.say(format!("Changed {}'s nickname to **{}**.", user.name, n)).await?;
        }
        _ => {
            member.edit(ctx, serenity::EditMember::new().nickname("")).await?;
            ctx.say(format!("Reset {}'s nickname.", user.name)).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_ROLES")]
async fn addrole(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to add role to"] user: serenity::User,
    #[description = "Role to add"] role: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let member = guild_id.member(ctx, user.id).await?;
    if member.roles.contains(&role.id) {
        ctx.say(format!("{} already has the role {}.", user.name, role.name)).await?;
        return Ok(());
    }
    member.add_role(ctx, role.id).await?;
    ctx.say(format!("Added {} to {}.", role.name, user.name)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_ROLES")]
async fn delrole(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "User to remove role from"] user: serenity::User,
    #[description = "Role to remove"] role: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let member = guild_id.member(ctx, user.id).await?;
    if !member.roles.contains(&role.id) {
        ctx.say(format!("{} doesn't have the role {}.", user.name, role.name)).await?;
        return Ok(());
    }
    member.remove_role(ctx, role.id).await?;
    ctx.say(format!("Removed {} from {}.", role.name, user.name)).await?;
    Ok(())
}
