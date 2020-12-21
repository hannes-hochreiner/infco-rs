use crate::service::Service;
use async_trait::async_trait;
use super::session::Session;

pub struct LocalService {
    session: Session
}

impl LocalService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(LocalService {session: Session::new()})
    }
}

#[async_trait]
impl Service for LocalService {
    async fn run(&mut self, command: String) -> Result<String, Box<dyn std::error::Error>> {
        /// TODO: add sudo
        self.session.run_command(&["bash", "-c", &*command], false).await
    }
}
