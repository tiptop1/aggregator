use std::collections::HashMap;

use crate::config::ReferenceFields;

use super::config::Config;
use indexmap::IndexMap;
use reqwest::Client;
use serde_json::{from_str, Value};
use serde_json_path::{ExactlyOneError, JsonPath, ParseError};
use thiserror::Error;

use std::thread;
use std::time::Duration;

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
        let service_endpoint = &service.endpoint;
        let mut request_builder = client.get(service_endpoint);
        if let Some(headers) = &service.headers {
            for (header, value) in headers.0.iter() {
                request_builder = request_builder.header(header, value);
            }
        };

        let response = request_builder.send().await?;
        let response_status = response.status();
        if response_status.is_success() {
            let category = &service.category;
            let json_content = &from_str(&response.text().await?)?;
            let reference_fields = evaluate_reference_fields(
                json_content,
                category,
                service_endpoint,
                &service.reference_fields,
            )?;
            for (field, path) in &(service.fields.0) {
                let mut new_path = path.clone();
                for (ref_field, ref_value) in &reference_fields {
                    let placeholder = format!("${{{}}}", ref_field);
                    if path.contains(&placeholder) {
                        new_path = new_path.replace(&placeholder, ref_value);
                    }
                }
                let node_str =
                    evaluate_json_path(json_content, category, service_endpoint, &new_path, false)?;
                aggregates.add(category.clone(), field.clone(), node_str);
            }
        } else {
            Err(AggregatorError::Aggregator(format!(
                "Request failed! Status: {}",
                response_status.as_str()
            )))?;
        }
        // Pause the current thread for 1 second to avoid rate limiting
        thread::sleep(Duration::from_secs(1));
    }
    Ok(aggregates)
}

fn evaluate_json_path(
    json_content: &Value,
    category: &String,
    service: &String,
    path: &String,
    ref_path: bool,
) -> Result<String, AggregatorError> {
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
        let path_type = if ref_path {
            "reference field path"
        } else {
            "field path"
        };
        Err(AggregatorError::Aggregator(format!(
            "Could not parse JSON for category '{}' and service '{}' with {} '{}'.",
            category, service, path_type, path
        )))
    }
}

fn evaluate_reference_fields(
    json_content: &Value,
    category: &String,
    service: &String,
    reference_fields: &Option<ReferenceFields>,
) -> Result<HashMap<String, String>, AggregatorError> {
    match reference_fields {
        Some(fields) => {
            let mut evaluated_fields = HashMap::new();
            for (field, path) in fields.0.iter() {
                let value = evaluate_json_path(json_content, category, service, path, true)?;
                evaluated_fields.insert(field.clone(), value);
            }
            Ok(evaluated_fields)
        }
        None => Ok(HashMap::new()),
    }
}
