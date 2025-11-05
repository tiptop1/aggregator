use super::config::Config;
use reqwest::get;
use serde_json::{json, Value};
use serde_json_path::{JsonPath, ParseError, ExactlyOneError};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AggregatorError {
    #[error("Failed to get content from url:: {0}")]
    Get(#[from] reqwest::Error),

    #[error("Failed to parse JSON: {0}")]
    Parse(#[from] ParseError),

    #[error("Failed to get one value from JSON: {0}")]
    ExactlyOne(#[from] ExactlyOneError),

}

#[derive(Debug)]
pub struct Aggregates {
    aggregates: HashMap<String, HashMap<String, String>>,
}

impl Aggregates {
    fn new() -> Self {
        Aggregates {
            aggregates: HashMap::new(),
        }
    }

    pub fn categories(&self) -> Vec<&String> {
        let mut categories = Vec::new();
        for s in self.aggregates.keys() {
            categories.push(s);
        }
        return categories;
    }

    pub fn fields(&self, category: &String) -> Option<&HashMap<String, String>> {
        let fields = self.aggregates.get(category)?;
        return Some(fields);
    }

    fn add(&mut self, category: String, field: String, value: String) {
        self.aggregates
            .entry(category)
            .and_modify(|e| {
                e.insert(field.clone(), value.clone());
                ()
            })
            .or_insert_with(|| {
                let mut new_fields = HashMap::new();
                new_fields.insert(field, value);
                new_fields
            });
    }
}

pub async fn aggregate_fields(config: &Config) -> Result<Aggregates, AggregatorError> {
    let mut aggregates = Aggregates::new();
    for service in &(config.service) {
        let endpoint = &(service.endpoint);
        let response = get(endpoint).await?;
        if response.status().is_success() {
            let category = &service.category;
            let json_content = json!(response.text().await?);
            for (field, path) in &(service.fields.0) {
                let path = JsonPath::parse(&path)?;
                let node = path.query(&json_content).exactly_one()?;
                let node_str = match node {
                    Value::Null => "Null".to_string(),
                    Value::Bool(value) => value.to_string(),
                    Value::Number(value) => value.to_string(),
                    Value::String(value) => value.clone(),
                    Value::Array(_value) => "JSON Array not supported!".to_string(),
                    Value::Object(_value) => "JSON Object not supported!".to_string()
                };
                aggregates.add(category.clone(), field.clone(), node_str);
            }
        } else {
            println!("Request failed! Status: {}", response.status());
        }
    }
    Ok(aggregates)
}
