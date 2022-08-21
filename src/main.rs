#[allow(clippy::pedantic)]
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
