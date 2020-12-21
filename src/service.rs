use async_trait::async_trait;

#[async_trait]
pub trait Service {
    async fn run(&mut self, command: String) -> Result<String, Box<dyn std::error::Error>>;
}
