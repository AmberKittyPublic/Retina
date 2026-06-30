use crate::types::{AppState, Error};
use poise::serenity_prelude as serenity;

pub fn commands() -> Vec<poise::Command<AppState, Error>> {
    vec![
        rps(), flip(), roll(), dadjoke(), cat(), dog(), pug(), github(), urban(),
        _8ball(), meme(), number(), roast(), yomama(), norris(), pokemon(),
        wouldyourather(), space(), translate(), weather(), remindme(), timer(),
        choose(), poll(), truth(), dare(), wyr(), nhie(), paranoia(), banroulette(),
    ]
}

async fn init_state(ctx: &poise::Context<'_, AppState, Error>) {
    let mut state = ctx.data().bot_state.write().await;
    state.commands_executed += 1;
}

#[poise::command(slash_command)]
pub async fn rps(ctx: poise::Context<'_, AppState, Error>, #[description = "Your choice"] choice: String) -> Result<(), Error> {
    init_state(&ctx).await;
    let choices = ["rock", "paper", "scissors"];
    let bot = choices[rand::random::<usize>() % 3];
    let user = choice.to_lowercase();
    if !choices.contains(&user.as_str()) {
        ctx.say("Choose rock, paper, or scissors.").await?;
        return Ok(());
    }
    let result = match (user.as_str(), bot) {
        (a, b) if a == b => "It's a tie!",
        ("rock", "scissors") | ("paper", "rock") | ("scissors", "paper") => "You win!",
        _ => "I win!",
    };
    ctx.say(format!("You chose {}, I chose {}. {}", user, bot, result)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn flip(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    init_state(&ctx).await;
    let side = if rand::random() { "Heads" } else { "Tails" };
    ctx.say(format!("🪙 {}", side)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn roll(ctx: poise::Context<'_, AppState, Error>, #[description = "Max value (default 100)"] max: Option<u64>) -> Result<(), Error> {
    init_state(&ctx).await;
    let max = max.unwrap_or(100).max(1);
    let n: u64 = rand::random::<u64>() % max + 1;
    ctx.say(format!("🎲 You rolled **{}** (1-{})", n, max)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn dadjoke(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get("https://icanhazdadjoke.com/")
        .header("Accept", "text/plain")
        .send().await;
    match resp {
        Ok(r) => { ctx.say(r.text().await.unwrap_or_default()).await?; }
        Err(_) => { ctx.say("Couldn't fetch a joke right now.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn cat(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get("https://api.thecatapi.com/v1/images/search").send().await;
    match resp {
        Ok(r) => {
            let json: Vec<serde_json::Value> = r.json().await.unwrap_or_default();
            let url = json.first().and_then(|v| v["url"].as_str()).unwrap_or("https://cdn2.thecatapi.com/images/0XYvRd7oD.jpg");
            ctx.say(url).await?;
        }
        Err(_) => { ctx.say("Couldn't fetch a cat picture.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn dog(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get("https://dog.ceo/api/breeds/image/random").send().await;
    match resp {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let url = json["message"].as_str().unwrap_or("");
            if url.is_empty() { ctx.say("Couldn't fetch a dog picture.").await?; }
            else { ctx.say(url).await?; }
        }
        Err(_) => { ctx.say("Couldn't fetch a dog picture.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn pug(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get("https://dog.ceo/api/breed/pug/images/random").send().await;
    match resp {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let url = json["message"].as_str().unwrap_or("");
            if url.is_empty() { ctx.say("Couldn't fetch a pug picture.").await?; }
            else { ctx.say(url).await?; }
        }
        Err(_) => { ctx.say("Couldn't fetch a pug picture.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn github(ctx: poise::Context<'_, AppState, Error>, #[description = "Repo (e.g. serenity-rs/serenity)"] repo: String) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get(format!("https://api.github.com/repos/{}", repo))
        .header("User-Agent", "RetinaBot").send().await;
    match resp {
        Ok(r) => {
            if !r.status().is_success() {
                ctx.say("Repository not found.").await?;
                return Ok(());
            }
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let embed = serenity::CreateEmbed::new()
                .title(json["full_name"].as_str().unwrap_or(&repo))
                .url(json["html_url"].as_str().unwrap_or(""))
                .description(json["description"].as_str().unwrap_or("No description"))
                .field("⭐ Stars", json["stargazers_count"].to_string(), true)
                .field("🍴 Forks", json["forks_count"].to_string(), true)
                .field("🐛 Issues", json["open_issues_count"].to_string(), true)
                .color(serenity::Colour::DARK_GREEN);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(_) => { ctx.say("Couldn't fetch repo info.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn urban(ctx: poise::Context<'_, AppState, Error>, #[description = "Term to define"] term: String) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get("https://api.urbandictionary.com/v0/define")
        .query(&[("term", &term)]).send().await;
    match resp {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let list = json["list"].as_array().and_then(|a| a.first()).cloned();
            match list {
                Some(entry) => {
                    let def = entry["definition"].as_str().unwrap_or("").chars().take(1000).collect::<String>();
                    let ex = entry["example"].as_str().unwrap_or("").chars().take(500).collect::<String>();
                    let embed = serenity::CreateEmbed::new()
                        .title(entry["word"].as_str().unwrap_or(&term))
                        .url(entry["permalink"].as_str().unwrap_or(""))
                        .description(def)
                        .field("Example", if ex.is_empty() { "None" } else { &ex }, false)
                        .footer(serenity::CreateEmbedFooter::new(format!("👍 {} | 👎 {}", entry["thumbs_up"], entry["thumbs_down"])))
                        .color(serenity::Colour::BLUE);
                    ctx.send(poise::CreateReply::default().embed(embed)).await?;
                }
                None => { ctx.say("No results found.").await?; }
            }
        }
        Err(_) => { ctx.say("Couldn't contact Urban Dictionary.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn _8ball(ctx: poise::Context<'_, AppState, Error>, #[description = "Your question"] question: String) -> Result<(), Error> {
    init_state(&ctx).await;
    let responses = [
        "It is certain.", "It is decidedly so.", "Without a doubt.", "Yes definitely.",
        "You may rely on it.", "As I see it, yes.", "Most likely.", "Outlook good.",
        "Yes.", "Signs point to yes.", "Reply hazy, try again.", "Ask again later.",
        "Better not tell you now.", "Cannot predict now.", "Concentrate and ask again.",
        "Don't count on it.", "My reply is no.", "My sources say no.",
        "Outlook not so good.", "Very doubtful.",
    ];
    let answer = responses[rand::random::<usize>() % responses.len()];
    ctx.say(format!("🎱 {} — {}", question, answer)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn meme(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get("https://meme-api.com/gimme").send().await;
    match resp {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let url = json["url"].as_str().unwrap_or("");
            let title = json["title"].as_str().unwrap_or("");
            if !url.is_empty() {
                let embed = serenity::CreateEmbed::new()
                    .title(title)
                    .image(url)
                    .color(serenity::Colour::PURPLE);
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
            } else {
                ctx.say("Couldn't fetch a meme.").await?;
            }
        }
        Err(_) => { ctx.say("Couldn't fetch a meme.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn number(ctx: poise::Context<'_, AppState, Error>, #[description = "Number (or 'random')"] number: Option<String>) -> Result<(), Error> {
    init_state(&ctx).await;
    let n = number.unwrap_or_else(|| "random".to_string());
    let client = reqwest::Client::new();
    let resp = client.get(format!("http://numbersapi.com/{}", n))
        .header("Accept", "text/plain").send().await;
    match resp {
        Ok(r) => { ctx.say(r.text().await.unwrap_or_default()).await?; }
        Err(_) => { ctx.say("Couldn't fetch a number fact.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn roast(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    init_state(&ctx).await;
    let roasts = [
        "You're not stupid; you just have bad luck thinking.",
        "I'd agree with you, but then we'd both be wrong.",
        "You're proof that evolution can go in reverse.",
        "I've seen salads more intimidating than you.",
        "You're like a cloud. When you disappear, it's a beautiful day.",
        "Somewhere a village is missing their idiot.",
        "You're not a complete idiot — some parts are missing.",
        "If I wanted to hear from an idiot, I'd watch your TikToks.",
        "Your brain is like a web browser — 15 tabs open and none of them loading.",
        "You bring everyone so much joy — when you leave.",
    ];
    let r = roasts[rand::random::<usize>() % roasts.len()];
    ctx.say(r).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn yomama(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    init_state(&ctx).await;
    let jokes = [
        "Yo mama's so fat, when she got on the scale it said 'I need your weight, not your phone number.'",
        "Yo mama's so ugly, she made One Direction split into No Direction.",
        "Yo mama's so dumb, she put airbags on her computer in case it crashed.",
        "Yo mama's so poor, I saw her throwing a penny into a wishing well and the well threw it back.",
        "Yo mama's so old, her birth certificate says 'Expired' on it.",
        "Yo mama's so fat, she was floating in the ocean and Spain claimed her as a new continent.",
        "Yo mama's so hairy, Bigfoot takes pictures of her.",
        "Yo mama's so short, you can see her feet in her driver's license photo.",
    ];
    let j = jokes[rand::random::<usize>() % jokes.len()];
    ctx.say(j).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn norris(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get("https://api.chucknorris.io/jokes/random").send().await;
    match resp {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            ctx.say(json["value"].as_str().unwrap_or("Chuck Norris fact unavailable.")).await?;
        }
        Err(_) => { ctx.say("Chuck Norris fact unavailable.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn pokemon(ctx: poise::Context<'_, AppState, Error>, #[description = "Pokemon name or ID"] pokemon: String) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get(format!("https://pokeapi.co/api/v2/pokemon/{}", pokemon.to_lowercase())).send().await;
    match resp {
        Ok(r) => {
            if !r.status().is_success() {
                ctx.say("Pokémon not found.").await?;
                return Ok(());
            }
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let name = json["name"].as_str().unwrap_or(&pokemon);
            let id = json["id"].as_u64().unwrap_or(0);
            let types: Vec<&str> = json["types"].as_array().map(|a| a.iter().filter_map(|t| t["type"]["name"].as_str()).collect()).unwrap_or_default();
            let sprite = json["sprites"]["other"]["official-artwork"]["front_default"].as_str()
                .or_else(|| json["sprites"]["front_default"].as_str()).unwrap_or("");
            let stats: String = json["stats"].as_array().map(|a| {
                a.iter().filter_map(|s| {
                    let name = s["stat"]["name"].as_str()?;
                    let val = s["base_stat"].as_u64()?;
                    Some(format!("{}: {}", name, val))
                }).collect::<Vec<_>>().join("\n")
            }).unwrap_or_default();
            let embed = serenity::CreateEmbed::new()
                .title(format!("#{} {}", id, name))
                .description(format!("Type: {}", types.join(", ")))
                .field("Base Stats", if stats.is_empty() { "Unknown".into() } else { stats }, false)
                .thumbnail(sprite)
                .color(serenity::Colour::from_rgb(255, 0, 0));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(_) => { ctx.say("Couldn't fetch Pokémon data.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn wouldyourather(ctx: poise::Context<'_, AppState, Error>,
    #[description = "Rating: pg, pg13, or r"] rating: Option<String>,
) -> Result<(), Error> {
    init_state(&ctx).await;
    wyr_impl(&ctx, rating).await
}

#[poise::command(slash_command)]
pub async fn space(ctx: poise::Context<'_, AppState, Error>) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get("http://api.open-notify.org/astros.json").send().await;
    match resp {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let count = json["number"].as_u64().unwrap_or(0);
            let people: Vec<&str> = json["people"].as_array().map(|a| a.iter().filter_map(|p| p["name"].as_str()).collect()).unwrap_or_default();
            ctx.say(format!("🌍 There are **{}** people in space right now:\n{}", count, people.join(", "))).await?;
        }
        Err(_) => { ctx.say("Couldn't fetch space data.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn translate(ctx: poise::Context<'_, AppState, Error>,
    #[description = "Text to translate"] text: String,
    #[description = "Target language code (e.g. es, fr, de)"] target: String,
) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get("https://api.mymemory.translated.net/get")
        .query(&[("q", &text), ("langpair", &format!("|{}", target))])
        .send().await;
    match resp {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let translated = json["responseData"]["translatedText"].as_str().unwrap_or("Translation unavailable.");
            ctx.say(format!("{} → *{}*", text, translated)).await?;
        }
        Err(_) => { ctx.say("Couldn't translate.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn weather(ctx: poise::Context<'_, AppState, Error>, #[description = "City name"] city: String) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let resp = client.get(format!("https://wttr.in/{}?format=%C+%t+%h+%w", city)).send().await;
    match resp {
        Ok(r) => {
            let text = r.text().await.unwrap_or_default().trim().to_string();
            if text.is_empty() {
                ctx.say("City not found.").await?;
            } else {
                ctx.say(format!("🌤 Weather in **{}**: {}", city, text)).await?;
            }
        }
        Err(_) => { ctx.say("Couldn't fetch weather.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn remindme(ctx: poise::Context<'_, AppState, Error>,
    #[description = "Time (e.g. 30s, 5m, 1h, 2d)"] duration: String,
    #[description = "Reminder message"] message: Option<String>,
) -> Result<(), Error> {
    init_state(&ctx).await;
    let secs = parse_duration(&duration).ok_or("Invalid duration. Use e.g. 30s, 5m, 1h, 2d")?;
    if secs < 10 { return Err("Duration must be at least 10 seconds.".into()); }
    if secs > 2592000 { return Err("Duration can't exceed 30 days.".into()); }
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    let remind_at = (chrono::Utc::now() + chrono::Duration::seconds(secs as i64)).to_rfc3339();
    let msg = message.clone().unwrap_or_default();
    let reminder = ctx.data().db.create_reminder(
        &ctx.author().id.to_string(),
        &guild_id.to_string(),
        &ctx.channel_id().to_string(),
        &msg,
        &remind_at,
    ).await?;
    ctx.say(format!("✅ Reminder set for {} (ID: {})", humantime_secs(secs), reminder.id)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn timer(ctx: poise::Context<'_, AppState, Error>,
    #[description = "Time (e.g. 30s, 5m, 1h)"] duration: String,
) -> Result<(), Error> {
    init_state(&ctx).await;
    let secs = parse_duration(&duration).ok_or("Invalid duration. Use e.g. 30s, 5m, 1h")?;
    if secs < 5 { return Err("Duration must be at least 5 seconds.".into()); }
    if secs > 86400 { return Err("Duration can't exceed 24 hours.".into()); }
    ctx.say(format!("⏰ Timer set for {}. I'll ping you when it's done!", humantime_secs(secs))).await?;
    let http = ctx.serenity_context().http.clone();
    let channel_id = ctx.channel_id();
    let author_id = ctx.author().id;
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
        let _ = channel_id.send_message(&http,
            serenity::CreateMessage::new().content(format!("⏰ <@{}> Timer finished!", author_id))).await;
    });
    Ok(())
}

#[poise::command(slash_command)]
pub async fn choose(ctx: poise::Context<'_, AppState, Error>,
    #[description = "Options separated by spaces (or in quotes)"] options: String,
) -> Result<(), Error> {
    init_state(&ctx).await;
    let items: Vec<&str> = options.split_whitespace().collect();
    if items.len() < 2 { return Err("Give me at least 2 options.".into()); }
    let choice = items[rand::random::<usize>() % items.len()];
    ctx.say(format!("I choose **{}**!", choice)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn poll(ctx: poise::Context<'_, AppState, Error>,
    #[description = "Question"] question: String,
    #[description = "Option 1"] option1: String,
    #[description = "Option 2"] option2: Option<String>,
    #[description = "Option 3"] option3: Option<String>,
    #[description = "Option 4"] option4: Option<String>,
) -> Result<(), Error> {
    init_state(&ctx).await;
    let mut options_str = format!("1️⃣ {}\n", option1);
    let mut options = vec![option1];
    for opt in [option2, option3, option4].into_iter().flatten() {
        let idx = options.len();
        if idx < 4 {
            options_str.push_str(&format!("{} {}\n", ["🇦", "🇧", "🇨", "🇩"][idx], opt));
            options.push(opt);
        }
    }
    let embed = serenity::CreateEmbed::new()
        .title("📊 Poll")
        .description(format!("**{}**\n\n{}", question, options_str))
        .color(serenity::Colour::BLUE);
    let msg = ctx.channel_id().send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    let reactions = ["1️⃣", "🇦", "🇧", "🇨", "🇩"];
    for i in 0..options.len() {
        let _ = msg.react(ctx, serenity::ReactionType::Unicode(reactions[i].to_string())).await;
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn truth(ctx: poise::Context<'_, AppState, Error>,
    #[description = "Rating: pg, pg13, or r"] rating: Option<String>,
) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let mut req = client.get("https://api.truthordarebot.xyz/v1/truth");
    if let Some(ref r) = rating {
        req = req.query(&[("rating", r)]);
    }
    match req.send().await {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let q = json["question"].as_str().unwrap_or("Couldn't fetch a truth question.");
            let rating_str = json["rating"].as_str().unwrap_or("?");
            ctx.say(format!("❓ **Truth** ({})\n{}", rating_str, q)).await?;
        }
        Err(_) => { ctx.say("Couldn't fetch a truth question.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn dare(ctx: poise::Context<'_, AppState, Error>,
    #[description = "Rating: pg, pg13, or r"] rating: Option<String>,
) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let mut req = client.get("https://api.truthordarebot.xyz/api/dare");
    if let Some(ref r) = rating {
        req = req.query(&[("rating", r)]);
    }
    match req.send().await {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let q = json["question"].as_str().unwrap_or("Couldn't fetch a dare question.");
            let rating_str = json["rating"].as_str().unwrap_or("?");
            ctx.say(format!("💪 **Dare** ({})\n{}", rating_str, q)).await?;
        }
        Err(_) => { ctx.say("Couldn't fetch a dare question.").await?; }
    }
    Ok(())
}

async fn wyr_impl(ctx: &poise::Context<'_, AppState, Error>, rating: Option<String>) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let mut req = client.get("https://api.truthordarebot.xyz/api/wyr");
    if let Some(ref r) = rating {
        req = req.query(&[("rating", r)]);
    }
    match req.send().await {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let q = json["question"].as_str().unwrap_or("Couldn't fetch a WYR question.");
            let rating_str = json["rating"].as_str().unwrap_or("?");
            ctx.say(format!("🤔 **Would You Rather** ({})\n{}", rating_str, q)).await?;
        }
        Err(_) => { ctx.say("Couldn't fetch a WYR question.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn wyr(ctx: poise::Context<'_, AppState, Error>,
    #[description = "Rating: pg, pg13, or r"] rating: Option<String>,
) -> Result<(), Error> {
    init_state(&ctx).await;
    wyr_impl(&ctx, rating).await
}

#[poise::command(slash_command)]
pub async fn nhie(ctx: poise::Context<'_, AppState, Error>,
    #[description = "Rating: pg, pg13, or r"] rating: Option<String>,
) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let mut req = client.get("https://api.truthordarebot.xyz/api/nhie");
    if let Some(ref r) = rating {
        req = req.query(&[("rating", r)]);
    }
    match req.send().await {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let q = json["question"].as_str().unwrap_or("Couldn't fetch an NHIE question.");
            let rating_str = json["rating"].as_str().unwrap_or("?");
            ctx.say(format!("🙊 **Never Have I Ever** ({})\n{}", rating_str, q)).await?;
        }
        Err(_) => { ctx.say("Couldn't fetch an NHIE question.").await?; }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn paranoia(ctx: poise::Context<'_, AppState, Error>,
    #[description = "Rating: pg, pg13, or r"] rating: Option<String>,
) -> Result<(), Error> {
    init_state(&ctx).await;
    let client = reqwest::Client::new();
    let mut req = client.get("https://api.truthordarebot.xyz/api/paranoia");
    if let Some(ref r) = rating {
        req = req.query(&[("rating", r)]);
    }
    match req.send().await {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let q = json["question"].as_str().unwrap_or("Couldn't fetch a paranoia question.");
            let rating_str = json["rating"].as_str().unwrap_or("?");
            ctx.say(format!("👀 **Paranoia** ({})\n{}", rating_str, q)).await?;
        }
        Err(_) => { ctx.say("Couldn't fetch a paranoia question.").await?; }
    }
    Ok(())
}

fn parse_duration(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(n) = s.strip_suffix('s').or_else(|| s.strip_suffix('S')) {
        n.parse::<u64>().ok()
    } else if let Some(n) = s.strip_suffix('m').or_else(|| s.strip_suffix('M')) {
        n.parse::<u64>().ok().map(|v| v * 60)
    } else if let Some(n) = s.strip_suffix('h').or_else(|| s.strip_suffix('H')) {
        n.parse::<u64>().ok().map(|v| v * 3600)
    } else if let Some(n) = s.strip_suffix('d').or_else(|| s.strip_suffix('D')) {
        n.parse::<u64>().ok().map(|v| v * 86400)
    } else {
        s.parse::<u64>().ok()
    }
}

#[poise::command(slash_command)]
pub async fn banroulette(
    ctx: poise::Context<'_, AppState, Error>,
    #[description = "Opponent to play against (turn-based)"] opponent: Option<serenity::User>,
) -> Result<(), Error> {
    init_state(&ctx).await;
    if ctx.guild_id().is_none() {
        ctx.say("This command can only be used in a server.").await?;
        return Ok(());
    };

    let author = ctx.author();
    let mut all_players = vec![author.id.get()];
    let mut desc;

    if let Some(ref opp) = opponent {
        if opp.id == author.id {
            ctx.say("You can't play against yourself!").await?;
            return Ok(());
        }
        if opp.bot {
            ctx.say("You can't play against a bot!").await?;
            return Ok(());
        }
        all_players.push(opp.id.get());
        let author_name = &author.name;
        let opp_name = &opp.name;
        desc = format!(
            "**{author_name}** vs **{opp_name}**!\n\n**1/6 odds** (default). Use **Spin** to crank it to **50/50**.\n\nIt's **{author_name}**'s turn first!"
        );
    } else {
        desc = String::from(
            "Russian roulette with a shadowban!\n\n6 chambers, 1 bullet. Keep pulling the trigger until you lose.\nIf you lose, you'll be shadowbanned for **1 minute**.",
        );
    }

    let game = crate::types::BanRouletteGame {
        players: all_players.clone(),
        current_turn: 0,
        odds_denominator: 6,
        active: true,
    };

    let msg = ctx.send(poise::CreateReply::default()
        .embed(create_br_embed(&desc, None))
        .components(br_components(&game))
    ).await?;

    let msg_id = msg.into_message().await?.id.to_string();
    {
        let mut bot_state = ctx.data().bot_state.write().await;
        bot_state.banroulette_games.insert(msg_id, game);
    }

    Ok(())
}

fn br_components(game: &crate::types::BanRouletteGame) -> Vec<serenity::CreateActionRow> {
    use serenity::builder::{CreateActionRow, CreateButton};
    if game.players.len() > 1 {
        vec![CreateActionRow::Buttons(vec![
            CreateButton::new("br_self")
                .label("🔫 Shoot Self")
                .style(serenity::ButtonStyle::Danger),
            CreateButton::new("br_opponent")
                .label("🎯 Shoot Opponent")
                .style(serenity::ButtonStyle::Danger),
            CreateButton::new("br_spin")
                .label("🔄 Spin Chamber")
                .style(serenity::ButtonStyle::Primary),
            CreateButton::new("br_cancel")
                .label("✋ Stop")
                .style(serenity::ButtonStyle::Secondary),
        ])]
    } else {
        vec![CreateActionRow::Buttons(vec![
            CreateButton::new("br_play")
                .label("🔫 Pull the Trigger")
                .style(serenity::ButtonStyle::Danger),
            CreateButton::new("br_cancel")
                .label("Cancel")
                .style(serenity::ButtonStyle::Secondary),
        ])]
    }
}

fn create_br_embed(desc: &str, footer: Option<&str>) -> serenity::CreateEmbed {
    let mut e = serenity::CreateEmbed::new()
        .title("🔫 Ban Roulette")
        .description(desc)
        .color(serenity::Colour::ORANGE);
    if let Some(f) = footer {
        e = e.footer(serenity::CreateEmbedFooter::new(f));
    }
    e
}

pub async fn handle_banroulette_component(
    ctx: &serenity::Context,
    state: &AppState,
    custom_id: &str,
    interaction: &serenity::ComponentInteraction,
) {
    let msg_id = interaction.message.id.to_string();
    let game = {
        let mut bot_state = state.bot_state.write().await;
        bot_state.banroulette_games.get(&msg_id).cloned()
    };
    let mut game = match game {
        Some(g) if g.active => g,
        _ => {
            let _ = interaction
                .create_response(&ctx.http, serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(serenity::CreateEmbed::new()
                            .title("🔫 Ban Roulette")
                            .description("This game is no longer active.")
                            .color(serenity::Colour::DARK_GREY))
                        .components(vec![]),
                ))
                .await;
            return;
        }
    };

    let user_id = interaction.user.id.get();
    let current_id = game.players[game.current_turn];

    if user_id != current_id {
        let msg = if game.players.len() > 1 {
            format!("It's <@{}>'s turn!", current_id)
        } else {
            "These buttons aren't for you!".to_string()
        };
        let _ = interaction
            .create_response(&ctx.http, serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content(msg)
                    .ephemeral(true),
            ))
            .await;
        return;
    }

    match custom_id {
        "br_play" => banroulette_play(ctx, state, &mut game, interaction).await,
        "br_self" => banroulette_shoot(ctx, state, &mut game, interaction, true).await,
        "br_opponent" => banroulette_shoot(ctx, state, &mut game, interaction, false).await,
        "br_spin" => banroulette_spin(ctx, state, &mut game, interaction).await,
        "br_cancel" => banroulette_cancel(ctx, state, &mut game, interaction).await,
        _ => {}
    }

    if game.active {
        let mut bot_state = state.bot_state.write().await;
        bot_state.banroulette_games.insert(msg_id, game);
    } else {
        let mut bot_state = state.bot_state.write().await;
        bot_state.banroulette_games.remove(&msg_id);
    }
}

async fn apply_shadowban(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    target_id: serenity::UserId,
) {
    let role_id = get_or_create_sb_role_serenity(ctx, guild_id).await;
    if let Some(role_id) = role_id {
        if let Ok(member) = guild_id.member(&ctx.http, target_id).await {
            if !member.roles.contains(&role_id) {
                let _ = member.add_role(&ctx.http, role_id).await;
            }
        }
        let http = ctx.http.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            if let Ok(member) = guild_id.member(&http, target_id).await {
                let _ = member.remove_role(&http, role_id).await;
            }
        });
    }
}

async fn banroulette_play(
    ctx: &serenity::Context,
    _state: &AppState,
    game: &mut crate::types::BanRouletteGame,
    interaction: &serenity::ComponentInteraction,
) {
    let guild_id = match interaction.guild_id {
        Some(id) => id,
        None => return,
    };
    let user_id = serenity::UserId::new(game.players[game.current_turn]);
    let player_name = &interaction.user.name;

    let lose = rand::random::<u8>() % 6 == 0;
    if lose {
        game.active = false;
        apply_shadowban(ctx, guild_id, user_id).await;

        let desc = format!("**BANG!** {player_name} lost and has been shadowbanned for **1 minute**.");
        let _ = interaction
            .create_response(&ctx.http, serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(create_br_embed(&desc, None).color(serenity::Colour::RED))
                    .components(vec![]),
            ))
            .await;
    } else {
        let desc = format!("**Click!** {player_name} is safe. Try again!");
        let _ = interaction
            .create_response(&ctx.http, serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(create_br_embed(&desc, Some("Keep pulling if you dare...")).color(serenity::Colour::DARK_GREEN))
                    .components(br_components(game)),
            ))
            .await;
    }
}

async fn banroulette_shoot(
    ctx: &serenity::Context,
    _state: &AppState,
    game: &mut crate::types::BanRouletteGame,
    interaction: &serenity::ComponentInteraction,
    shoot_self: bool,
) {
    let guild_id = match interaction.guild_id {
        Some(id) => id,
        None => return,
    };
    let shooter_id = serenity::UserId::new(game.players[game.current_turn]);
    let target_idx = if shoot_self {
        game.current_turn
    } else {
        (game.current_turn + 1) % game.players.len()
    };
    let target_id = serenity::UserId::new(game.players[target_idx]);
    let player_name = &interaction.user.name;
    let target_name = if shoot_self {
        player_name.clone()
    } else {
        let guild = ctx.cache.guild(guild_id);
        let member = guild.and_then(|g| g.members.get(&target_id).cloned());
        member
            .as_ref()
            .map(|m| m.display_name().to_string())
            .unwrap_or_else(|| "opponent".to_string())
    };

    let odds = game.odds_denominator;
    let lose = rand::random::<u8>() % odds == 0;
    game.odds_denominator = 6;

    if lose {
        game.active = false;
        apply_shadowban(ctx, guild_id, target_id).await;

        let verb = if shoot_self { "lost" } else { "shot" };
        let desc = format!(
            "**BANG!** {target_name} {verb} and has been shadowbanned for **1 minute**."
        );
        let _ = interaction
            .create_response(&ctx.http, serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(create_br_embed(&desc, None).color(serenity::Colour::RED))
                    .components(vec![]),
            ))
            .await;
    } else {
        game.current_turn = (game.current_turn + 1) % game.players.len();
        let next_id = game.players[game.current_turn];
        let desc = format!(
            "**Click!** {player_name} pulled the trigger on {target_name} and got lucky."
        );
        let _ = interaction
            .create_response(&ctx.http, serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(
                        create_br_embed(
                            &desc,
                            Some(&format!("It's <@{}>'s turn. (odds reset to 1/6)", next_id)),
                        )
                        .color(serenity::Colour::DARK_GREEN),
                    )
                    .components(br_components(game)),
            ))
            .await;
    }
}

async fn banroulette_spin(
    ctx: &serenity::Context,
    _state: &AppState,
    game: &mut crate::types::BanRouletteGame,
    interaction: &serenity::ComponentInteraction,
) {
    let outcome = rand::random::<u8>() % 3;
    let (new_odds, outcome_desc) = match outcome {
        0 => (6, "Nothing changed. Odds stay at **1/6**.".to_string()),
        1 => (2, "Odds increased to **50/50**! 🔥".to_string()),
        _ => (10, "Odds decreased to **1/10**! 😅".to_string()),
    };

    game.odds_denominator = new_odds;
    game.current_turn = (game.current_turn + 1) % game.players.len();
    let next_id = game.players[game.current_turn];
    let player_name = &interaction.user.name;

    let desc = format!("**{player_name}** spun the chamber!\n{outcome_desc}");
    let _ = interaction
        .create_response(&ctx.http, serenity::CreateInteractionResponse::UpdateMessage(
            serenity::CreateInteractionResponseMessage::new()
                .embed(
                    create_br_embed(
                        &desc,
                        Some(&format!("It's <@{}>'s turn!", next_id)),
                    )
                    .color(serenity::Colour::PURPLE),
                )
                .components(br_components(game)),
        ))
        .await;
}

async fn banroulette_cancel(
    ctx: &serenity::Context,
    _state: &AppState,
    game: &mut crate::types::BanRouletteGame,
    interaction: &serenity::ComponentInteraction,
) {
    game.active = false;
    let _ = interaction
        .create_response(&ctx.http, serenity::CreateInteractionResponse::UpdateMessage(
            serenity::CreateInteractionResponseMessage::new()
                .embed(serenity::CreateEmbed::new()
                    .title("Ban Roulette")
                    .description("Cancelled. 🫡")
                    .color(serenity::Colour::DARK_GREY))
                .components(vec![]),
        ))
        .await;
}

async fn get_or_create_sb_role_serenity(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
) -> Option<serenity::RoleId> {
    use serenity::builder::EditRole;
    use serenity::model::channel::{ChannelType, PermissionOverwrite, PermissionOverwriteType};
    use serenity::model::Permissions;

    let existing = ctx.cache.guild(guild_id).and_then(|g| {
        g.roles
            .values()
            .find(|r| r.name == "Shadow Banned")
            .map(|r| r.id)
    });
    if let Some(role_id) = existing {
        return Some(role_id);
    }

    let role = guild_id
        .create_role(
            &ctx.http,
            EditRole::new()
                .name("Shadow Banned")
                .permissions(Permissions::empty())
                .colour(serenity::Colour::from_rgb(30, 30, 30))
                .hoist(false)
                .mentionable(false),
        )
        .await
        .ok()?;

    if let Ok(channels) = guild_id.channels(&ctx.http).await {
        for (channel_id, channel) in &channels {
            if channel.kind == ChannelType::Category {
                continue;
            }
            let (allow, deny) = match channel.kind {
                ChannelType::Voice | ChannelType::Stage => (
                    Permissions::empty(),
                    Permissions::SPEAK | Permissions::STREAM,
                ),
                _ => (
                    Permissions::empty(),
                    Permissions::SEND_MESSAGES
                        | Permissions::ADD_REACTIONS
                        | Permissions::CREATE_PUBLIC_THREADS
                        | Permissions::CREATE_PRIVATE_THREADS
                        | Permissions::SEND_MESSAGES_IN_THREADS,
                ),
            };
            let overwrite = PermissionOverwrite {
                allow,
                deny,
                kind: PermissionOverwriteType::Role(role.id),
            };
            let _ = channel_id.create_permission(&ctx.http, overwrite).await;
        }
    }

    Some(role.id)
}

fn humantime_secs(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let mut parts = vec![];
    if d > 0 { parts.push(format!("{}d", d)); }
    if h > 0 { parts.push(format!("{}h", h)); }
    if m > 0 { parts.push(format!("{}m", m)); }
    if s > 0 || parts.is_empty() { parts.push(format!("{}s", s)); }
    parts.join(" ")
}
