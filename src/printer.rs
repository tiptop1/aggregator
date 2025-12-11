use super::aggregator::Aggregates;
use super::config::Config;

pub trait Printer {
    fn print(&self, aggregates: &Aggregates);

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
                },
                _ => (),
            }
        }
        str
    }
}

pub struct StdoutPrinter;

impl Printer for StdoutPrinter {
    fn print(&self, aggregates: &Aggregates) {
        print!("{}", Printer::to_string(self, aggregates));
    }
}

pub struct SmtpPrinter<'a> {
    config: &'a Config,
}

impl<'a> SmtpPrinter<'a> {
    pub fn new(config: &'a Config) -> SmtpPrinter {
        SmtpPrinter { config }
    }
}

impl<'a> Printer for SmtpPrinter<'a> {
    fn print(&self, aggregates: &Aggregates) {
        print!("Not implemented yet!");
    }
}
