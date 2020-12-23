use crate::Service;
use serde_json::Value;

pub async fn run(context: &mut Box<dyn Service>, config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    context.run(config["command"].as_str().unwrap().to_string()).await
}
