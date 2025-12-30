use super::config::Config;
use indexmap::IndexMap;
use reqwest::Client;
use serde_json::{from_str, Value};
use serde_json_path::{ExactlyOneError, JsonPath, ParseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AggregatorError {
    #[error("Failed to get content from url.")]
    Get(#[from] reqwest::Error),

    #[error("Failed to parse JSON.")]
    Parse(#[from] ParseError),

    #[error("Failed to get one value from JSON.")]
    ExactlyOne(#[from] ExactlyOneError),

    #[error("Failed to creae JSON.")]
    Json(#[from] serde_json::Error),

    #[error("Aggregator error: {0}.")]
    Aggregator(String),
}

#[derive(Debug)]
pub struct Aggregates {
    aggregates: IndexMap<String, IndexMap<String, String>>,
}

impl Aggregates {
    fn new() -> Self {
        Aggregates {
            aggregates: IndexMap::new(),
        }
    }

    pub fn categories(&self) -> Vec<&String> {
        let mut categories = Vec::new();
        for s in self.aggregates.keys() {
            categories.push(s);
        }
        return categories;
    }

    pub fn fields(&self, category: &String) -> Option<&IndexMap<String, String>> {
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
                let mut new_fields = IndexMap::new();
                new_fields.insert(field, value);
                new_fields
            });
    }
}

pub async fn aggregate_fields(config: &Config) -> Result<Aggregates, AggregatorError> {
    let mut aggregates = Aggregates::new();
    for service in &(config.services) {
        let client = Client::new();
        let mut request_builder = client.get(&service.endpoint);
        if let Some(headers) = &service.headers {
            for (header, value) in headers.0.iter() {
                request_builder = request_builder.header(header, value);
            }
        };

        let response = request_builder.send().await?;
        let response_status = response.status();
        if response_status.is_success() {
            let category = &service.category;
            let json_content = from_str(&response.text().await?)?;
            for (field, path) in &(service.fields.0) {
                let node_str = evaluate_json_path(&json_content, path)?;
                aggregates.add(category.clone(), field.clone(), node_str);
            }
        } else {
            Err(AggregatorError::Aggregator(format!(
                "Request failed! Status: {}",
                response_status.as_str()
            )))?;
        }
    }
    Ok(aggregates)
}

fn evaluate_json_path(json_content: &Value, path: &String) -> Result<String, AggregatorError> {
    let json_path = JsonPath::parse(&path)?;
    let values = json_path.query(&json_content).all();
    if values.len() > 0 {
        let node = match values.first() {
            Some(value) => value,
            None => &&Value::Null,
        };
        match node {
            Value::Null => Ok("Null".to_string()),
            Value::Bool(value) => Ok(value.to_string()),
            Value::Number(value) => Ok(value.to_string()),
            Value::String(value) => Ok(value.clone()),
            Value::Array(_value) => Err(AggregatorError::Aggregator(
                "JSON Array not supported!".to_string(),
            )),
            Value::Object(_value) => Err(AggregatorError::Aggregator(
                "JSON Object not supported!".to_string(),
            )),
        }
    } else {
        Err(AggregatorError::Aggregator(
            "Could not parse JSON.".to_string(),
        ))
    }
}
