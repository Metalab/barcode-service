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
use anyhow::Error;
use barcode_service::protocol::{Date, Request, Response};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_postgres::{Config, NoTls};
use tokio_serde_cbor::Codec;
use tokio_util::codec::Decoder;

/*
create table drinks (
    id serial primary key,
    date varchar(8) not null,
    ean varchar(255) not null,
    count int not null,
    unique (date, ean)
);
 */

#[derive(Parser)]
struct BarcodeParams {
    start_year: u16,
    start_month: u8,
    start_day: u8,
    end_year: u16,
    end_month: u8,
    end_day: u8,
    /// IP and port to connect to
    #[clap(short, long, default_value = "127.0.0.1:2348")]
    server: String,
    #[clap(long)]
    db_host: String,
    #[clap(long)]
    db_port: u16,
    #[clap(long)]
    db_name: String,
    #[clap(long)]
    db_user: String,
    #[clap(long)]
    db_password: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let config = BarcodeParams::parse();

    let stream = TcpStream::connect(config.server).await?;
    let codec: Codec<Response, Request> = Codec::new();
    let mut framed = codec.framed(stream);

    let request = Request {
        start: Date {
            year: config.start_year,
            month: config.start_month,
            day: config.start_day,
        },
        end: Date {
            year: config.end_year,
            month: config.end_month,
            day: config.end_day,
        },
    };

    framed.send(request).await?;

    let (response, _) = framed.into_future().await;

    if response.is_none() {
        log::error!("Can't read response");
        return Ok(());
    }
    let response = response.unwrap()?;
    let mut db_config = Config::new();
    db_config
        .host(&config.db_host)
        .port(config.db_port)
        .user(&config.db_user)
        .password(config.db_password.as_bytes())
        .dbname(&config.db_name);

    let (mut client, connection) = db_config.connect(NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let transaction = client.transaction().await?;

    log::info!("Inserting {} rows", response.0.len());
    let mut row_count = 0;
    for row in response.0 {
        row_count += transaction
            .execute(
                "INSERT INTO drinks (date, ean, count) VALUES (?, ?, ?)",
                &[&row.date.to_string(), &row.code, &row.count],
            )
            .await?;
    }
    log::info!("{row_count} rows added.");

    transaction.commit().await?;

    Ok(())
}
