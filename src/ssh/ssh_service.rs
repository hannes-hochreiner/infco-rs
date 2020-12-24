use crate::service::Service;
use async_trait::async_trait;
use tokio::sync::{oneshot, mpsc};
use tokio::runtime::Runtime;
use tokio::task;
use super::{error::SshError, session::Session};

enum Command {
    Command { command: String },
    FileRead { path: String },
    FileWrite { path: String, data: Vec<u8> },
}

type CommandResponse = Option<Vec<u8>>;

pub struct SshService {
    cmd_tx: mpsc::Sender<(Command, oneshot::Sender<CommandResponse>)>,
}

impl SshService {
    pub fn get_server_fingerprint(host: &str, user: &str) -> Result<String, Box<dyn std::error::Error>> {
        Session::get_server_fingerprint(host, user)
    }

    pub fn new(host: String, user: String, hash: String) -> Result<Self, Box<dyn std::error::Error>> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<(Command, oneshot::Sender<CommandResponse>)>(100);
        tokio::task::spawn_blocking(|| {
            let rt  = Runtime::new().unwrap();
            let local = task::LocalSet::new();
    
            local.block_on(&rt, async move {
                let mut session = Session::new_with_host_user_hash(&*host, &*user, &*hash).unwrap();
                while let Some((cmd, response)) = cmd_rx.recv().await {
                    match cmd {
                        /// TODO: Error handling
                        Command::Command{command} => {
                            response.send(Some(Vec::from(session.run_command(&*command).unwrap().as_bytes()))).unwrap();
                        },
                        Command::FileRead{path} => {
                            let res = session.file_read(path);
                            response.send(Some(res.unwrap())).unwrap()
                        },
                        Command::FileWrite{path, data} => {
                            session.file_write(path, data).unwrap();
                            response.send(None).unwrap()
                        },
                        // Command::Terminate => {
                        //     response.send(None).unwrap();
                        //     break
                        // }
                    }
                }
            });
        });
        
        Ok(SshService {
            cmd_tx: cmd_tx,
        })
    }

    async fn send_command(&mut self, command: Command) -> Result<CommandResponse, Box<dyn std::error::Error>> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx.send((command, resp_tx)).await.ok();
        resp_rx.await.map_err(|e| e.into())
    }
}

#[async_trait]
impl Service for SshService {
    async fn run(&mut self, command: String) -> Result<String, Box<dyn std::error::Error>> {
        match self.send_command(Command::Command{command: command}).await {
            Ok(Some(res)) => Ok(String::from_utf8(res)?),
            Ok(None) => Err(SshError::new(String::from("unexpected result")).into()),
            Err(err) => Err(err)
        }
    }

    async fn file_read(&mut self, path: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match self.send_command(Command::FileRead{path: path}).await {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(SshError::new(String::from("no data read")).into()),
            Err(err) => Err(err)
        }
    }

    async fn file_write(&mut self, path: String, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        match self.send_command(Command::FileWrite{path: path, data: data}).await {
            Ok(Some(_)) => Err(SshError::new(String::from("received unexpected result while writing data")).into()),
            Ok(None) => Ok(()),
            Err(err) => Err(err)
        }
    }
}
