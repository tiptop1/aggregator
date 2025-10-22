use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::Error;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
   pub service: Vec<Service>
}


#[derive(Debug, Deserialize)]
pub struct Fields(pub HashMap<String, String>);

#[derive(Debug, Deserialize)]
pub struct Service {
    pub endpoint: String,
    pub category: String,
    pub fields: Fields
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    Io(#[from] Error),

    #[error("Failed to parse TOML configuration: {0}")]
    Parse(#[from] toml::de::Error),
}

pub fn read_configuration(file_path: &PathBuf) -> Result<Config, ConfigError> {
    let config_file_content = fs::read_to_string(file_path)?;
    let config = toml::from_str(&config_file_content)?;
    Ok(config)
}
