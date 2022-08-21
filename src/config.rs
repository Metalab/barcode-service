use std::path::{Path, PathBuf};

use anyhow::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub listen: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Barcode {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub logging: log4rs::config::RawConfig,
    pub barcode: Barcode,
}

pub async fn load(path: impl AsRef<Path>) -> Result<Config, Error> {
    let config = tokio::fs::read_to_string(path).await?;
    Ok(toml::from_str(&config)?)
}
