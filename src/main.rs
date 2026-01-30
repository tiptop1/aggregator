use aggregator::config::{read_configuration, ConfigError};
use aggregator::aggregator::{aggregate_fields, AggregatorError};
use aggregator::printer::{Printer, PrinterError, SmtpPrinter};
use std::path::PathBuf;
use thiserror::Error;
use tokio;
use std::env::{self, args};


#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Failed to get configuration: {0}")]
    Config(#[from] ConfigError),

    #[error("Failed to aggregate fields: {0}")]
    Aggregator(#[from] AggregatorError),
   
    #[error("Failed to send message via SMTP: {0}")]
    Printer(#[from] PrinterError)
}

const CONFIG_FILE: &str = "config.toml";

#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    // Try to find config file in OUT_DIR (useful in development time),
    // if not found try to use config file from application argument,
    // if not found try to use config file from current directory
    let config_file = match env::var_os("OUT_DIR") {
        Some(path) => PathBuf::from(path).join(CONFIG_FILE),
        None => {
            let args: Vec<String> = args().collect();
            if args.len() > 1 {
                // Get from index 1, because in 0 is the application path
                if let Some(arg1) = args.get(1) {
                    PathBuf::from(arg1)
                } else {
                    PathBuf::from(CONFIG_FILE)
                }
            } else {
                PathBuf::from(CONFIG_FILE)
            }
        }
    };
    let config = read_configuration(&config_file)?;
    let aggregates = aggregate_fields(&config).await;
    let printer = SmtpPrinter::new(&config.smtp_printer);
    let _ = printer.print(&aggregates)?;
    Ok(())
}
