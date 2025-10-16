use leprechaun::config::{read_configuration, ConfigError};
use std::path::PathBuf;


fn main() -> Result<(), ConfigError> {
    let mut path = PathBuf::new();
    path.push("config.toml");
    let config = read_configuration(&path)?;
    print!("{:?}", config);
    Ok(())
}
