use std::error::Error;
use std::process::Stdio;
use rpassword;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use super::error::LocalError;

pub struct Session {
}

impl Session {
    pub fn new() -> Session {
        Session {}
    }

    pub async fn run_command(&mut self, command: &[&str], sudo: bool) -> Result<String, Box<dyn Error>> {
        let mut cmd = match sudo {
            true => {
                let mut cmd = Command::new("sudo");
                let mut args = vec!["-S"];
    
                args.append(&mut Vec::from(command));
                cmd.args(args);
                cmd
            },
            false => {
                let mut cmd = Command::new(command[0]);

                cmd.args(&command[1..]);
                cmd
            }
        };

        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
    
        let mut child = cmd.spawn()?;
        let mut stderr = child.stderr.take().unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let hndl = tokio::spawn(async move {
            child.wait_with_output().await.unwrap()
        });
    
        let mut data = [0; 15];
        stderr.read(&mut data).await?;
        let mut err_string = std::string::String::from_utf8(data.to_vec())?;
    
        if err_string.starts_with("[sudo] password") {
            let pass = rpassword::prompt_password_stdout("enter sudo password: ")? + "\n";
            stdin.write(pass.as_bytes()).await?;
        }
    
        let mut err_string2 = String::new();
        stderr.read_to_string(&mut err_string2).await?;
    
        err_string += err_string2.as_str();
        let output = hndl.await?;
    
        match output.status.success() {
            true => Ok(String::from_utf8(output.stdout)?),
            false => Err(Box::new(LocalError::new(err_string)))
        }
    }
}
