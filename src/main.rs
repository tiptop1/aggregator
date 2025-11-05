use aggregator::config::{read_configuration, ConfigError};
use aggregator::aggregator::{aggregate_fields, AggregatorError};
use aggregator::printer::stdout_print;
use std::path::PathBuf;
use thiserror::Error;
use tokio;


#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Failed to get configuration: {0}")]
    Config(#[from] ConfigError),

    #[error("Failed to aggregate fields: {0}")]
    Aggregator(#[from] AggregatorError)

}


#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    let mut path = PathBuf::new();
    path.push("config.toml");
    let config = read_configuration(&path)?;
    let aggregates = aggregate_fields(&config).await?;
    stdout_print(&aggregates);
    Ok(())
}
