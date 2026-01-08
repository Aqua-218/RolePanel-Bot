use std::env;
use std::fmt;

#[derive(Clone)]
pub struct Config {
    pub discord_token: String,
    pub database_url: String,
    pub health_port: u16,
    pub database_max_connections: u32,
    pub error_webhook_url: Option<String>,
    // Bot info (customizable via env)
    pub bot_name: String,
    pub bot_description: String,
    pub bot_developer_id: String,
    pub bot_github_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let discord_token = env::var("DISCORD_TOKEN")
            .map_err(|_| ConfigError::Missing("DISCORD_TOKEN"))?;

        if discord_token.is_empty() {
            return Err(ConfigError::Invalid("DISCORD_TOKEN", "must not be empty"));
        }

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::Missing("DATABASE_URL"))?;

        if database_url.is_empty() {
            return Err(ConfigError::Invalid("DATABASE_URL", "must not be empty"));
        }

        let health_port = env::var("HEALTH_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| ConfigError::Invalid("HEALTH_PORT", "must be a valid port number"))?;

        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .map_err(|_| {
                ConfigError::Invalid("DATABASE_MAX_CONNECTIONS", "must be a positive integer")
            })?;

        // Error webhook URL is optional
        let error_webhook_url = env::var("ERROR_WEBHOOK_URL").ok().filter(|s| !s.is_empty());

        // Bot info (with defaults)
        let bot_name = env::var("BOT_NAME")
            .unwrap_or_else(|_| "Role Panel Bot".to_string());
        let bot_description = env::var("BOT_DESCRIPTION")
            .unwrap_or_else(|_| "2025年10月同期会向けに作られたロールパネルBot".to_string());
        let bot_developer_id = env::var("BOT_DEVELOPER_ID")
            .unwrap_or_else(|_| "1340270150217236501".to_string());
        let bot_github_url = env::var("BOT_GITHUB_URL")
            .unwrap_or_else(|_| "https://github.com/Aqua-218/RolePanel-Bot".to_string());

        Ok(Self {
            discord_token,
            database_url,
            health_port,
            database_max_connections,
            error_webhook_url,
            bot_name,
            bot_description,
            bot_developer_id,
            bot_github_url,
        })
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("discord_token", &"[REDACTED]")
            .field("database_url", &"[REDACTED]")
            .field("health_port", &self.health_port)
            .field("database_max_connections", &self.database_max_connections)
            .field("error_webhook_url", &self.error_webhook_url.as_ref().map(|_| "[SET]"))
            .field("bot_name", &self.bot_name)
            .field("bot_description", &self.bot_description)
            .field("bot_developer_id", &self.bot_developer_id)
            .field("bot_github_url", &self.bot_github_url)
            .finish()
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Missing(&'static str),
    Invalid(&'static str, &'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(name) => write!(f, "missing environment variable: {}", name),
            ConfigError::Invalid(name, reason) => {
                write!(f, "invalid environment variable {}: {}", name, reason)
            }
        }
    }
}

impl std::error::Error for ConfigError {}
