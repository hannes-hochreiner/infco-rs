use crate::Service;
use serde_json::Value;
use super::error::TaskError;
use tokio::fs::{write, read};

pub async fn run(context: &mut Box<dyn Service>, config: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let local_path = config["localPath"].as_str().ok_or(TaskError::new("error reading local path"))?;
    let context_path = config["contextPath"].as_str().ok_or(TaskError::new("error reading context path"))?;

    match config["direction"].as_str() {
        Some("contextToLocal") => {
            Ok(write(local_path.to_string(), context.file_read(context_path.to_string()).await?).await?)
        },
        Some("localToContext") => {
            Ok(context.file_write(context_path.to_string(), read(local_path.to_string()).await?).await?)
        },
        Some(dir) => Err(TaskError::new(&*format!("unknown direction \"{}\" given", dir)).into()),
        None => Err(TaskError::new("no direction given").into())
    }
}
