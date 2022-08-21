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
