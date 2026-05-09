use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub prefix: String,
    pub owner_ids: Vec<u64>,
    pub modules: ModulesConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            prefix: "!".to_string(),
            owner_ids: vec![],
            modules: ModulesConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        Config::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModulesConfig {
    pub moderation: bool,
    pub auto_mod: bool,
    pub logging: bool,
    pub welcome: bool,
    pub custom_commands: bool,
    pub reaction_roles: bool,
    pub tickets: bool,
    pub xp: bool,
    pub scheduling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuildConfig {
    pub guild_id: String,
    pub modules: ModulesConfig,
    pub auto_mod: AutoModConfig,
    pub welcome: WelcomeConfig,
}

// ── Auto-mod ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoModConfig {
    pub enabled: bool,
    #[serde(default)]
    pub rules: Vec<AutoModRule>,
    #[serde(default)]
    pub channel_whitelist: Vec<String>,
    #[serde(default)]
    pub channel_blacklist: Vec<String>,
    #[serde(default)]
    pub role_whitelist: Vec<String>,
    #[serde(default)]
    pub role_blacklist: Vec<String>,
}

impl Default for AutoModConfig {
    fn default() -> Self {
        AutoModConfig {
            enabled: false,
            channel_whitelist: vec![],
            channel_blacklist: vec![],
            role_whitelist: vec![],
            role_blacklist: vec![],
            rules: vec![
                AutoModRule::new("spam", false, "warn")
                    .with_max_messages(5).with_window_seconds(5),
                AutoModRule::new("caps", false, "delete")
                    .with_caps_percent(70),
                AutoModRule::new("links", false, "delete"),
                AutoModRule::new("mentions", false, "warn")
                    .with_max_mentions(5),
                AutoModRule::new("emotes", false, "delete")
                    .with_max_emotes(5),
                AutoModRule::new("banned_words", false, "warn"),
                AutoModRule::new("max_length", false, "delete")
                    .with_max_length(2000),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoModRule {
    pub rule_type: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_action")]
    pub action: String,
    #[serde(default)]
    pub action_duration_minutes: Option<u32>,
    #[serde(default)]
    pub caps_percent: Option<u32>,
    #[serde(default)]
    pub max_messages: Option<u32>,
    #[serde(default)]
    pub window_seconds: Option<u32>,
    #[serde(default)]
    pub max_mentions: Option<u32>,
    #[serde(default)]
    pub max_emotes: Option<u32>,
    #[serde(default)]
    pub max_length: Option<usize>,
    #[serde(default)]
    pub banned_words: Vec<String>,
}

// ── Welcome / Goodbye ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeConfig {
    pub enabled: bool,
    pub welcome_channel_id: String,
    pub goodbye_channel_id: String,
    pub welcome_message: String,
    pub goodbye_message: String,
}

impl Default for WelcomeConfig {
    fn default() -> Self {
        WelcomeConfig {
            enabled: false,
            welcome_channel_id: String::new(),
            goodbye_channel_id: String::new(),
            welcome_message: "Welcome {user} to {guild}!".to_string(),
            goodbye_message: "Goodbye {user}, we'll miss you!".to_string(),
        }
    }
}

fn default_action() -> String {
    "delete".to_string()
}

impl AutoModRule {
    pub fn new(rule_type: &str, enabled: bool, action: &str) -> Self {
        AutoModRule {
            rule_type: rule_type.to_string(),
            enabled,
            action: action.to_string(),
            action_duration_minutes: None,
            caps_percent: None,
            max_messages: None,
            window_seconds: None,
            max_mentions: None,
            max_emotes: None,
            max_length: None,
            banned_words: vec![],
        }
    }

    pub fn with_caps_percent(mut self, v: u32) -> Self { self.caps_percent = Some(v); self }
    pub fn with_max_messages(mut self, v: u32) -> Self { self.max_messages = Some(v); self }
    pub fn with_window_seconds(mut self, v: u32) -> Self { self.window_seconds = Some(v); self }
    pub fn with_max_mentions(mut self, v: u32) -> Self { self.max_mentions = Some(v); self }
    pub fn with_max_emotes(mut self, v: u32) -> Self { self.max_emotes = Some(v); self }
    pub fn with_max_length(mut self, v: usize) -> Self { self.max_length = Some(v); self }
}

// ── Application settings (from config.toml) ──

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub discord: DiscordSettings,
    #[serde(default)]
    pub web: WebSettings,
    #[serde(default)]
    pub database: DatabaseSettings,
}

impl Settings {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let settings: Settings = toml::from_str(&content)?;
        Ok(settings)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordSettings {
    pub token: String,
    pub client_id: String,
    pub client_secret: String,
    pub owner_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebSettings {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

impl Default for WebSettings {
    fn default() -> Self {
        WebSettings { port: 3000, host: default_host() }
    }
}

fn default_port() -> u16 { 3000 }
fn default_host() -> String { "0.0.0.0".to_string() }

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    #[serde(default = "default_db_url")]
    pub url: String,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        DatabaseSettings { url: default_db_url() }
    }
}

fn default_db_url() -> String { "sqlite:retina.db".to_string() }
