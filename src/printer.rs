use super::aggregator::Aggregates;
use super::config::SmtpPrinterConfig;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrinterError {
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),

    #[error("LettreError: {0}")]
    LettreError(#[from] lettre::error::Error),

    #[error("SmtpError: {0}")]
    SmtpError(#[from] lettre::transport::smtp::Error),

    #[error("AddressError: {0}")]
    AddressError(#[from] lettre::address::AddressError),
}

pub trait Printer {
    fn print(&self, aggregates: &Aggregates) -> Result<(), PrinterError>;

    fn to_string(&self, aggregates: &Aggregates) -> String {
        let mut str = String::new();
        for category in aggregates.categories() {
            str.push_str("***** ");
            str.push_str(category);
            str.push_str(" *****\n");
            match aggregates.fields(category) {
                Some(fields) => {
                    for (k, v) in fields.iter() {
                        str.push_str(k);
                        str.push_str(": ");
                        str.push_str(v);
                        str.push('\n');
                    }
                }
                _ => (),
            }
        }
        str
    }
}

pub struct StdoutPrinter;

impl Printer for StdoutPrinter {
    fn print(&self, aggregates: &Aggregates) -> Result<(), PrinterError> {
        print!("{}", Printer::to_string(self, aggregates));
        Ok(())
    }
}

pub struct SmtpPrinter<'a> {
    config: &'a SmtpPrinterConfig,
}

impl<'a> SmtpPrinter<'a> {
    pub fn new(config: &SmtpPrinterConfig) -> SmtpPrinter {
        SmtpPrinter { config }
    }
}

impl<'a> Printer for SmtpPrinter<'a> {
    fn print(&self, aggregates: &Aggregates) -> Result<(), PrinterError> {
        let email = Message::builder()
            .from(self.config.from_email.parse()?)
            .to(self.config.to_emails.parse()?)
            .subject("[Aggregator] Report")
            .body(Printer::to_string(self, aggregates))?;

        let creds = Credentials::new(self.config.username.clone(), self.config.password.clone());

        let mailer = SmtpTransport::starttls_relay(&self.config.server)?
            .credentials(creds)
            .build();

        match mailer.send(&email) {
            Ok(_) => println!("✅ Email sent successfully!"),
            Err(e) => eprintln!("❌ Could not send email: {:?}", e),
        }

        Ok(())
    }

    /*
        use lettre::{
        Message, SmtpTransport, Transport,
        transport::smtp::authentication::Credentials,
    };

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        // 1. Build the Email Message
        let email = Message::builder()
            // The sender's address (required)
            .from("Sender Name <sender@example.com>".parse()?)
            // The recipient's address (required)
            .to("Recipient Name <recipient@example.com>".parse()?)
            // The email subject
            .subject("Test Email from Rust/Lettre")
            // The body content (plain text)
            .body(String::from("Hello, this email was sent using the lettre library in Rust."))?;

        // 2. Define Credentials and SMTP Server Details
        // NOTE: Use an App Password for services like Gmail, NOT your main password.
        let username = "sender@example.com";
        let password = "your_smtp_password"; // Use the App Password
        let relay_host = "smtp.example.com"; // e.g., smtp.gmail.com for Gmail, or your relay

        let creds = Credentials::new(username.to_string(), password.to_string());

        // 3. Create the SMTP Transport (Mailer)
        // `starttls_relay` connects on port 587 and upgrades to TLS
        let mailer = SmtpTransport::starttls_relay(relay_host)?
            .credentials(creds)
            .build();

        // 4. Send the Email
        match mailer.send(&email) {
            Ok(_) => println!("✅ Email sent successfully!"),
            Err(e) => eprintln!("❌ Could not send email: {:?}", e),
        }

        Ok(())
    }
         */
}
