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

#![allow(clippy::pedantic)]
use std::path::PathBuf;

use anyhow::Error;
use clap::Parser;
use tokio::{net::TcpListener, task::spawn};

mod config;
mod connection;
pub mod protocol;

#[derive(Parser)]
struct BarcodeParams {
    /// The path to the configuration file
    #[clap(short, long, default_value = "config.toml")]
    pub config: PathBuf,
    /// IP and port to listen on (overrides the config file)
    #[clap(short, long)]
    pub listen: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let params = BarcodeParams::parse();
    let config = config::load(&params.config).await?;
    log4rs::config::init_raw_config(config.logging)?;

    let addr = params.listen.unwrap_or(config.server.listen);
    let listener = TcpListener::bind(&addr).await?;
    log::info!("Listening on {addr}...");

    loop {
        let path = config.barcode.path.clone();
        let (socket, addr) = listener.accept().await?;
        spawn(async move {
            if let Err(err) = connection::handle(socket, path).await {
                log::error!("Connection error from {addr:?}: {err:?}");
            }
        });
    }
}
