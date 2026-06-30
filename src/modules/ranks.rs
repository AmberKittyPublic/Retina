use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;

pub struct RanksModule;

impl RanksModule {
    pub fn new() -> Self { RanksModule }
}

pub fn commands() -> Vec<poise::Command<AppState, Error>> {
    vec![addrank(), delrank(), rank_cmd(), ranks()]
}

#[poise::command(slash_command, required_permissions = "MANAGE_ROLES")]
async fn addrank(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Rank name"] name: String,
    #[description = "Role ID or mention"] role: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    ctx.data().db.add_self_role(&guild_id.to_string(), &name, &role.id.to_string()).await?;
    ctx.say(format!("✅ Rank **{}** added ({}).", name, role.name)).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_ROLES")]
async fn delrank(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Rank name to delete"] name: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    ctx.data().db.remove_self_role(&guild_id.to_string(), &name).await?;
    ctx.say(format!("✅ Rank **{}** deleted.", name)).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn rank_cmd(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Rank name to join/leave"] name: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let role = ctx.data().db.get_self_role_by_name(&guild_id.to_string(), &name).await?
        .ok_or_else(|| format!("Rank **{}** not found.", name))?;

    let role_id: serenity::RoleId = role.role_id.parse().map_err(|_| "Invalid role ID.")?;
    let mut member = guild_id.member(ctx, ctx.author().id).await?;

    if member.roles.contains(&role_id) {
        member.remove_role(ctx, role_id).await?;
        ctx.say(format!("Left rank **{}**.", name)).await?;
    } else {
        member.add_role(ctx, role_id).await?;
        ctx.say(format!("Joined rank **{}**.", name)).await?;
    }
    Ok(())
}

#[poise::command(slash_command)]
async fn ranks(
    ctx: poise::Context<'_, AppState, Error>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let list = ctx.data().db.list_self_roles(&guild_id.to_string()).await?;
    if list.is_empty() {
        ctx.say("No ranks configured.").await?;
        return Ok(());
    }
    let embed = serenity::CreateEmbed::new()
        .title("Available Ranks")
        .description(list.iter().map(|r| format!("**{}** → <@&{}>", r.name, r.role_id)).collect::<Vec<_>>().join("\n"))
        .color(serenity::Colour::BLUE);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
