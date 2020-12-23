use crate::Service;
use serde_json::Value;
use super::error::TaskError;
use tokio::fs::write;

pub async fn run(context: &mut Box<dyn Service>, config: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let local_path = config["localPath"].as_str().ok_or(TaskError::new(String::from("error reading local path")))?;
    let context_path = config["contextPath"].as_str().ok_or(TaskError::new(String::from("error reading context path")))?;

    match config["direction"].as_str() {
        Some("contextToLocal") => {
            Ok(write(local_path.to_string(), context.file_read(context_path.to_string()).await?).await?)
        },
        Some("localToContext") => {
            Ok(write(context_path.to_string(), context.file_read(local_path.to_string()).await?).await?)
        },
        Some(dir) => Err(Box::new(TaskError::new(format!("unknown direction \"{}\" given", dir)))),
        None => Err(Box::new(TaskError::new("no direction given".into())))
    }
}
