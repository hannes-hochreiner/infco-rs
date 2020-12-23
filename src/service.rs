use async_trait::async_trait;

#[async_trait]
pub trait Service {
    async fn run(&mut self, command: String) -> Result<String, Box<dyn std::error::Error>>;
    async fn file_read(&mut self, path: String) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    async fn file_write(&mut self, path: String, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>>;
}
