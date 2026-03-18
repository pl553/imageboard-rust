use std::path::PathBuf;
use serde::Deserialize;
use config::{Config as ConfigBuilder, ConfigError, Environment, File};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiry_hours: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    pub images_path: PathBuf,
    pub thumbnails_path: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let cfg = ConfigBuilder::builder()
            // base defaults
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("database.max_connections", 10)?
            .set_default("jwt.expiry_hours", 24)?
            .set_default("storage.images_path", "./uploads/images")?
            .set_default("storage.thumbnails_path", "./uploads/thumbs")?
            // optional config.toml overrides defaults
            .add_source(File::with_name("config").required(false))
            // env vars override everything
            // APP_SERVER__PORT=9000, APP_DATABASE__URL=postgres://... etc
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .ignore_empty(true),
            )
            // DATABASE_URL and JWT_SECRET as top-level env vars
            .set_override_option("database.url", std::env::var("DATABASE_URL").ok())?
            .set_override_option("jwt.secret", std::env::var("JWT_SECRET").ok())?
            .build()?;

        cfg.try_deserialize()
    }
}

impl ServerConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
