use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub jwt_secret: String,

    #[serde(default = "default_jwt_expiry")]
    pub jwt_expiry_secs: u64,

    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_jwt_expiry() -> u64 { 3600 }
fn default_port() -> u16 { 3000 }

static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    pub fn load() -> &'static Config {
        CONFIG.get_or_init(|| {
            dotenvy::dotenv().ok();
            envy::from_env::<Config>().expect("Failed to load config from environment")
        })
    }
}