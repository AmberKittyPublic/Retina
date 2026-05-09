mod commands;
mod events;
mod modules;
mod config;
mod web;
mod database;
mod types;
mod cli;

use types::{AppState, BotState};
use cli::Cli;
use clap::Parser;
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();

    let settings = config::Settings::load(&args.config)
        .unwrap_or_else(|e| {
            eprintln!("Failed to load config from '{}': {}", args.config, e);
            eprintln!("Create a config.toml file (see config.example.toml) or pass --config <path>");
            std::process::exit(1);
        });

    println!("Loaded config from: {}", args.config);

    let db = database::Database::init(&settings.database.url).await?;
    println!("Database initialized");

    let mut bot_config = config::Config::load();
    if let Some(owner_id) = &settings.discord.owner_id {
        if let Ok(id) = owner_id.parse::<u64>() {
            bot_config.owner_ids.push(id);
        }
    }

    println!("Loaded config: modules - moderation:{}, auto_mod:{}, logging:{}",
        bot_config.modules.moderation, bot_config.modules.auto_mod, bot_config.modules.logging);

    let state = AppState {
        bot_state: Arc::new(RwLock::new(BotState {
            started_at: Some(std::time::SystemTime::now()),
            ..Default::default()
        })),
        web_state: Arc::new(RwLock::new(Default::default())),
        config: Arc::new(RwLock::new(bot_config)),
        settings: settings.clone(),
        db,
        spam_tracker: Arc::new(RwLock::new(HashMap::new())),
    };

    modules::init_modules(&state).await;

    let state_clone = state.clone();
    let token = settings.discord.token.clone();

    tokio::spawn(async move {
        if let Err(e) = start_bot(state_clone, token).await {
            eprintln!("Bot error: {}", e);
        }
    });

    let sched_state = state.clone();
    tokio::spawn(async move {
        // Give short delay for bot to connect before checking giveaways
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        crate::modules::giveaway::check_expired_giveaways(&sched_state).await;
    });

    let sched_state2 = state.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        crate::modules::scheduling::run_scheduler(sched_state2).await;
    });

    let reminder_state = state.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        crate::modules::reminders::run_reminder_checker(reminder_state).await;
    });

    if let Err(e) = web::start(state, settings.web.host.clone(), settings.web.port).await {
        eprintln!("Web dashboard error: {}", e);
    }

    Ok(())
}

async fn start_bot(state: AppState, token: String) -> Result<(), Box<dyn std::error::Error>> {
    use poise::serenity_prelude as serenity;

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::commands(),
            event_handler: |ctx, event, framework, state| {
                Box::pin(async move {
                    events::handle_event(ctx, &event, framework, state).await
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(state)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;
    Ok(())
}
