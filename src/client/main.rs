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
use barcode_service::protocol::{Request, Response};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use time::{format_description, Date, OffsetDateTime};
use tokio::net::TcpStream;
use tokio_postgres::{Config, NoTls};
use tokio_serde_cbor::Codec;
use tokio_util::codec::Decoder;

/*
create table drinks (
    id serial primary key,
    date date not null,
    ean varchar(255) not null,
    count int not null,
    unique (date, ean)
);
 */

#[derive(Parser)]
struct BarcodeParams {
    /// Request data starting from this date (inclusive). Format: YYYY-MM-DD, defaults to yesterday
    #[clap(value_parser = parse_date)]
    start: Option<Date>,
    /// Request data ending on this date (inclusive). Format: YYYY-MM-DD, defaults to yesterday
    #[clap(value_parser = parse_date)]
    end: Option<Date>,
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

fn parse_date(s: &str) -> Result<Date, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let format = format_description::parse("[year]-[month]-[day]")?;
    Ok(Date::parse(s, &format)?)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let config = BarcodeParams::parse();

    let stream = TcpStream::connect(config.server).await?;
    let codec: Codec<Response, Request> = Codec::new();
    let mut framed = codec.framed(stream);

    let request = Request {
        start: config.start.unwrap_or_else(|| {
            OffsetDateTime::now_local()
                .unwrap_or_else(|_| OffsetDateTime::now_utc())
                .date()
                .previous_day()
                .unwrap()
        }),
        end: config.end.unwrap_or_else(|| {
            OffsetDateTime::now_local()
                .unwrap_or_else(|_| OffsetDateTime::now_utc())
                .date()
                .previous_day()
                .unwrap()
        }),
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
                "INSERT INTO drinks (date, ean, count) VALUES ($1, $2, $3) ON CONFLICT (date, ean) DO UPDATE SET count = $3",
                &[&row.date, &row.code, &(row.count as i32)],
            )
            .await?;
    }
    log::info!("{row_count} rows added.");

    transaction.commit().await?;

    Ok(())
}
