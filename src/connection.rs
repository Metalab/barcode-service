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

use std::{io::ErrorKind, path::PathBuf};

use anyhow::{anyhow, Error};
use barcode_service::protocol::{Date, Request, Response, Row};
use futures_util::{SinkExt, StreamExt};
use tokio::{fs::read_dir, net::TcpStream};
use tokio_serde_cbor::Codec;
use tokio_util::codec::Decoder;

pub async fn handle(stream: TcpStream, path: impl Into<PathBuf>) -> Result<(), Error> {
    let codec: Codec<Request, Response> = Codec::new();
    let mut framed = codec.framed(stream);
    let path = path.into();

    if let Some(request) = framed.next().await {
        let request = request?;
        log::debug!("Received request: {request:?}");

        let mut response = Response(Vec::new());
        {
            let mut date = Date {
                year: request.start.year,
                month: request.start.month,
                day: request.start.day,
            };
            while date <= request.end {
                let mut dir_path = path.clone();
                dir_path.push(date.to_string());

                log::debug!("Reading path {dir_path:?}");
                match read_dir(&dir_path).await {
                    Ok(mut dir) => {
                        while let Some(entry) = dir.next_entry().await? {
                            let code = entry.file_name().into_string().map_err(|_| {
                                anyhow!("Unable to read file {:?}", entry.file_name())
                            })?;
                            let count_bytes = tokio::fs::read(entry.path()).await?;
                            if let Ok(count_bytes) = count_bytes.try_into() {
                                let count = u32::from_le_bytes(count_bytes);
                                response.0.push(Row { date, code, count });
                            } else {
                                return Err(anyhow!(
                                    "File {:?} does not contain 4 bytes",
                                    entry.file_name()
                                ));
                            }
                        }
                    }
                    Err(err) if err.kind() != ErrorKind::NotFound => {
                        return Err(err.into());
                    }
                    Err(_) => {}
                }
                date = date.next();
            }
        }
        framed.send(response).await?;
    }

    Ok(())
}
