use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;

pub fn commands() -> Vec<poise::Command<AppState, Error>> {
    vec![avatar(), whois(), serverinfo(), membercount(), randomcolor(), invite(), prefix(), emotes()]
}

#[poise::command(slash_command)]
pub async fn avatar(ctx: poise::Context<'_, AppState, Error>, #[description = "User"] user: Option<serenity::User>) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let url = u.face();
    let embed = serenity::CreateEmbed::new()
        .title(format!("{}'s Avatar", u.name))
        .image(url)
        .color(serenity::Colour::BLUE);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn whois(ctx: poise::Context<'_, AppState, Error>, #[description = "User"] user: Option<serenity::User>) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let member = guild_id.member(ctx, u.id).await.ok();
    let roles = member.as_ref().map(|m| m.roles.len()).unwrap_or(0);
    let joined = member.as_ref().and_then(|m| m.joined_at.map(|t| format!("<t:{}:R>", t.unix_timestamp()))).unwrap_or_else(|| "Unknown".into());
    let created = format!("<t:{}:R>", u.created_at().unix_timestamp());
    let embed = serenity::CreateEmbed::new()
        .title(u.name.clone())
        .thumbnail(u.face())
        .field("ID", u.id.to_string(), true)
        .field("Created", created, true)
        .field("Joined", joined, true)
        .field("Roles", roles.to_string(), true)
        .field("Bot", if u.bot { "Yes" } else { "No" }, true)
        .color(serenity::Colour::BLUE);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn serverinfo(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let (name, icon_url, owner_id, member_count, channels_len, roles_len, banner_url) = {
        let guild = ctx.cache().guild(guild_id).ok_or("Guild not found")?;
        (guild.name.clone(), guild.icon_url().unwrap_or_default(), guild.owner_id,
         guild.member_count, guild.channels.len(), guild.roles.len(), guild.banner_url())
    };
    let created = format!("<t:{}:R>", guild_id.created_at().unix_timestamp());
    let embed = serenity::CreateEmbed::new()
        .title(name)
        .thumbnail(icon_url)
        .field("ID", guild_id.to_string(), true)
        .field("Owner", format!("<@{}>", owner_id), true)
        .field("Created", created, true)
        .field("Members", &member_count.to_string(), true)
        .field("Channels", &channels_len.to_string(), true)
        .field("Roles", &roles_len.to_string(), true)
        .color(serenity::Colour::GOLD);
    let embed = if let Some(banner) = banner_url {
        embed.image(banner)
    } else {
        embed
    };
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn membercount(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let count = {
        let guild = ctx.cache().guild(guild_id).ok_or("Guild not found")?;
        guild.member_count
    };
    let embed = serenity::CreateEmbed::new()
        .title("Member Count")
        .field("Total", count.to_string(), true)
        .color(serenity::Colour::DARK_GREEN);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn randomcolor(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let r = rand::random::<u8>();
    let g = rand::random::<u8>();
    let b = rand::random::<u8>();
    let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
    let embed = serenity::CreateEmbed::new()
        .title("Random Color")
        .description(hex)
        .color(serenity::Colour::from_rgb(r, g, b));
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn invite(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let client_id = &ctx.data().settings.discord.client_id;
    let url = format!("https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot%20applications.commands", client_id);
    let embed = serenity::CreateEmbed::new()
        .title("Invite Retina Bot")
        .description(format!("[Click here to invite]({})", url))
        .color(serenity::Colour::BLUE);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn prefix(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let prefix = ctx.data().config.read().await.prefix.clone();
    ctx.say(format!("My prefix is `{}`", prefix)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn emotes(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let emoji_data = {
        let guild = ctx.cache().guild(guild_id).ok_or("Guild not found")?;
        guild.emojis.values().map(|e| (e.name.clone(), e.id, e.animated)).collect::<Vec<_>>()
    };
    if emoji_data.is_empty() {
        ctx.say("This server has no custom emojis.").await?;
        return Ok(());
    }
    let emoji_str: String = emoji_data.chunks(20).map(|chunk| {
        chunk.iter().map(|(name, id, animated)| {
            if *animated { format!("<a:{}:{}>", name, id) }
            else { format!("<:{}:{}>", name, id) }
        }).collect::<Vec<_>>().join(" ")
    }).collect::<Vec<_>>().join("\n");
    let embed = serenity::CreateEmbed::new()
        .title(format!("Server Emojis ({})", emoji_data.len()))
        .description(emoji_str)
        .color(serenity::Colour::GOLD);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
