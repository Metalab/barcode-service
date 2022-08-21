// Copyright [2022] Andreas Monitzer

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
