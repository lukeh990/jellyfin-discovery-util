/*
 * main.rs
 * Copyright (c) 2025 Luke Harding
 * This code is licensed under a MIT license.
 * See the file "LICENSE" in the root of this project.
 */

mod config;

use log::info;
use std::error;
use systemd_journal_logger::{JournalLog, connected_to_journal};

fn main() {
    enable_logging();

    info!("Starting Jellyfin Discovery Utility.")
}

fn enable_logging() {
    if connected_to_journal() {
        if systemd_logger().is_err() {
            env_logger::init();
        }
    } else {
        env_logger::init();
    }
}

fn systemd_logger() -> Result<(), Box<dyn error::Error>> {
    let journal_log = JournalLog::new()?;

    journal_log
        .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
        .install()?;

    Ok(())
}
