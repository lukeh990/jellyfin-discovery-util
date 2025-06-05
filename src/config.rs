/*
 * config.rs
 * Copyright (c) 2025 Luke Harding
 * This code is licensed under a MIT license.
 * See the file "LICENSE" in the root of this project.
 */

use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::{
    env, fs, io,
    path::{self, PathBuf},
};
use thiserror::Error;

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub id: String,
    pub name: String,
}

impl ServerConfig {
    pub fn new(url: &str, id: &str, name: &str) -> ServerConfig {
        ServerConfig {
            url: url.to_string(),
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    version: String,
    pub port: u16,
    pub server: Vec<ServerConfig>,
}

impl Config {
    pub fn new(port: u16, servers: Vec<ServerConfig>) -> Config {
        Config {
            version: "1".to_string(),
            port,
            server: servers,
        }
    }
}

pub fn read_config() -> Result<Config, ConfigError> {
    trace!("Config File Version: v1");

    let file_path = match env::var("JDU_CONF") {
        Ok(val) => path::absolute(&val)?,
        Err(_) => {
            debug!("Using default config file name.");
            path::absolute("discover.toml")?
        }
    };

    if file_path.exists() {
        trace!("Using existing file");

        let contents = &fs::read_to_string(file_path)?;

        let config: Config = toml::from_str(contents)?;

        Ok(config)
    } else {
        warn!(
            "Creating Sample Config! It can be found at: {}",
            file_path.display()
        );

        create_sample_config(&file_path)
    }
}

fn create_sample_config(file_path: &PathBuf) -> Result<Config, ConfigError> {
    let config = Config::new(
        7359,
        vec![ServerConfig::new(
            "http://jellyfin-test.local",
            "CHANGEME",
            "Test Jellyfin Server",
        )],
    );

    let config_str: String = toml::to_string(&config)?;

    fs::write(file_path, config_str)?;

    Ok(config)
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Error While Deserializing Struct")]
    Derserialize(#[from] toml::de::Error),
    #[error("Error While Serializing Struct")]
    Serialize(#[from] toml::ser::Error),
    #[error("Error While Preforming IO Operation")]
    Io(#[from] io::Error),
}
