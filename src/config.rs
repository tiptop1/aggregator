use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::Error;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
   pub services: Vec<Service>,
   pub smtp_printer: SmptPrinterConfig
}


#[derive(Debug, Deserialize)]
pub struct Fields(pub HashMap<String, String>);

#[derive(Debug, Deserialize)]
pub struct Headers(pub HashMap<String, String>);


#[derive(Debug, Deserialize)]
pub struct Service {
    pub category: String,
    pub endpoint: String,
    pub headers: Option<Headers>,
    pub fields: Fields
}

#[derive(Debug, Deserialize)]
pub struct SmptPrinterConfig  {
    pub server: String,
    pub port: i32,
    pub security: Option<String>,
    pub login: String,
    pub password: String,
    pub from_email: String,
    pub to_emails: String
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
