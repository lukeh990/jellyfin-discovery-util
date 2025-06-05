/*
 * main.rs
 * Copyright (c) 2025 Luke Harding
 * This code is licensed under a MIT license.
 * See the file "LICENSE" in the root of this project.
 */

mod config;

use bytes::{Bytes, BytesMut};
use config::ServerConfig;
use env_logger::Env;
use log::{LevelFilter, debug, error, info, trace};
use serde::Serialize;
use std::{
    error,
    net::{SocketAddr, UdpSocket},
    process,
};
use systemd_journal_logger::{JournalLog, connected_to_journal};

fn main() {
    enable_logging();

    info!("Starting Jellyfin Discovery Utility. v1.0.3");

    let config = handle_error(config::read_config(), 100);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let socket = handle_error(UdpSocket::bind(addr), 101);

    debug!("Created socket and bound to 0.0.0.0:{}", config.port);

    let preconstructed_response = preconstruct_response(config.server);
    trace!(
        "Pre-generated resonse complete. {} servers loaded",
        preconstructed_response.len()
    );

    loop {
        let mut buf = BytesMut::zeroed(22);
        let (amt, src) = handle_error(socket.recv_from(&mut buf), 102);

        trace!("Injested {} bytes from {}", amt, &src);

        if &buf[..] == b"who is JellyfinServer?" || &buf[..] == b"Who is JellyfinServer?" {
            trace!("Valid Discovery");

            for response in &preconstructed_response {
                handle_error(socket.send_to(response, src), 104);
                trace!("Sent response");
            }
        } else {
            trace!(
                "Invalid Discovery: {:?}",
                &buf
            );
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Response {
    address: String,
    id: String,
    name: String,
    endpoint_address: Option<()>,
}

fn preconstruct_response(server_config: Vec<ServerConfig>) -> Vec<Bytes> {
    let mut output = vec![];

    for config in server_config {
        let response = Response {
            address: config.url,
            id: config.id,
            name: config.name,
            endpoint_address: None,
        };

        let response_vec = handle_error(serde_json::to_vec(&response), 103);
        output.push(Bytes::from(response_vec));
    }

    output
}

fn handle_error<X, Y>(res: Result<X, Y>, code: i32) -> X
where
    Y: error::Error,
{
    match res {
        Ok(x) => x,
        Err(e) => {
            error!("{}", dbg!(e));

            process::exit(code);
        }
    }
}

fn enable_logging() {
    if connected_to_journal() {
        if systemd_logger().is_err() {
            env_logger();
        }
    } else {
        env_logger();
    }
}

fn env_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}

fn systemd_logger() -> Result<(), Box<dyn error::Error>> {
    let journal_log = JournalLog::new()?;

    journal_log
        .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
        .install()?;

    log::set_max_level(LevelFilter::Info);

    Ok(())
}
