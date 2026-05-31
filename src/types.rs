use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use crate::database::Database;

#[derive(Clone)]
pub struct AppState {
    pub bot_state: Arc<RwLock<BotState>>,
    pub web_state: Arc<RwLock<WebState>>,
    pub config: Arc<RwLock<crate::config::Config>>,
    pub settings: crate::config::Settings,
    pub db: Database,
    pub spam_tracker: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

#[derive(Debug, Clone, Default)]
pub struct GuildInfo {
    pub name: String,
    pub owner_id: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BanRouletteGame {
    pub players: Vec<u64>,
    pub current_turn: usize,
    pub odds_denominator: u8,
    pub active: bool,
}

#[derive(Default)]
pub struct BotState {
    pub commands_executed: u64,
    pub started_at: Option<std::time::SystemTime>,
    pub bot_guilds: HashSet<String>,
    pub guild_info: HashMap<String, GuildInfo>,
    pub banroulette_games: HashMap<String, BanRouletteGame>,
}

#[derive(Default)]
pub struct WebState {
    pub visits: u64,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
