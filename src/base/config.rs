use crate::base::args::Args;
use clap::Parser;
use serde::Deserialize;
use std::{error::Error, fs, path::PathBuf, sync::OnceLock};
use toml;
use url::Url;

static ARGS: OnceLock<Args> = OnceLock::new();

impl Args {
    pub fn get() -> &'static Self {
        ARGS.get_or_init(|| Args::parse())
    }
}

#[derive(Deserialize, Clone)]
pub struct ConfigService {
    pub name: String,
    pub sock_path: PathBuf,
    pub enabled: bool,
    #[serde(default = "default_mon_interval")]
    pub mon_interval_sec: u64,
}

fn default_mon_interval() -> u64 {
    30
}

#[derive(Deserialize)]
pub struct ConfigAdmin {
    pub backup_chat_id: i64,
    pub backup_time: String, // "HH:MM" local time
}

#[derive(Deserialize)]
pub struct ConfigTokens {
    #[serde(default)]
    pub admin_tokens: Vec<String>,
    #[serde(default)]
    pub backup_tokens: Vec<String>,
}

#[derive(Deserialize)]
pub struct ConfigBot {
    pub en: bool,
    pub webhook_url: Option<Url>,
    pub token: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub sock_path: PathBuf,
    pub kv_path: PathBuf,
    #[serde(default)]
    pub services: Vec<ConfigService>,
    #[serde(default)]
    pub admin: Option<ConfigAdmin>,
    #[serde(default)]
    pub tokens: Option<ConfigTokens>,
    pub bot: ConfigBot,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    fn load() -> Result<Self, Box<dyn Error>> {
        let args = Args::get();
        let content = fs::read_to_string(&args.config)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn get() -> &'static Self {
        CONFIG.get_or_init(|| {
            Self::load().unwrap_or_else(|err| panic!("Failed to load config: {err}"))
        })
    }
}
