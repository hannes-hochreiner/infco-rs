use crate::service::Service;
use async_trait::async_trait;
use tokio::sync::{oneshot, mpsc};
use tokio::runtime::Runtime;
use tokio::task;
use super::session::Session;

enum Command {
    Command { command: String },
    Terminate
}

pub struct SshService {
    cmd_tx: mpsc::Sender<(Command, oneshot::Sender<String>)>,
}

impl SshService {
    pub fn new(host: String, user: String) -> Result<Self, Box<dyn std::error::Error>> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<(Command, oneshot::Sender<String>)>(100);
        tokio::task::spawn_blocking(|| {
            let rt  = Runtime::new().unwrap();
            let local = task::LocalSet::new();
    
            local.block_on(&rt, async move {
                let mut session = Session::new_with_host_user(&*host, &*user).unwrap();
    
                while let Some((cmd, response)) = cmd_rx.recv().await {
                    match cmd {
                        /// TODO: Error handling
                        Command::Command{command} => {
                            response.send(session.run_command(&*command).unwrap()).unwrap();
                        },
                        Command::Terminate => {
                            response.send(String::new()).unwrap();
                            break
                        }
                    }
                }
            });
        });
        
        Ok(SshService {
            cmd_tx: cmd_tx,
        })
    }

    async fn send_command(&mut self, command: Command) -> Result<String, Box<dyn std::error::Error>> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx.send((command, resp_tx)).await.ok().unwrap();
        let res = resp_rx.await;
        let (resp2_tx, resp2_rx) = oneshot::channel();
        self.cmd_tx.send((Command::Terminate, resp2_tx)).await.ok().unwrap();
        let _ = resp2_rx.await.unwrap();

        match res {
            Ok(result) => Ok(result),
            Err(err) => Err(Box::new(err))
        }
    }
}

#[async_trait]
impl Service for SshService {
    async fn run(&mut self, command: String) -> Result<String, Box<dyn std::error::Error>> {
        self.send_command(Command::Command{command: command}).await
    }

    async fn file_read(&mut self, path: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(vec![0])
    }

    async fn file_write(&mut self, path: String, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
