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

use anyhow::Error;
use barcode_service::protocol::{Date, Request, Response};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_serde_cbor::Codec;
use tokio_util::codec::Decoder;

#[derive(Parser)]
struct BarcodeParams {
    start_year: u16,
    start_month: u8,
    start_day: u8,
    end_year: u16,
    end_month: u8,
    end_day: u8,
    #[clap(short, long, default_value = "127.0.0.1:2348")]
    server: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let config = BarcodeParams::parse();

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

    let stream = TcpStream::connect(config.server).await?;
    let codec: Codec<Response, Request> = Codec::new();
    let mut framed = codec.framed(stream);

    framed.send(request).await?;

    let (response, _) = framed.into_future().await;
    println!("{response:?}");

    Ok(())
}
