use std::error::Error;
use std::fmt::{Display, Formatter, Result, Debug};

#[derive(Debug)]
pub struct SshError {
    description : String,
}

impl SshError {
    pub fn new(description: &str) -> SshError {
        SshError {
            description: description.into()
        }
    }
}

impl Error for SshError {

}

impl Display for SshError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.description)
    }
}
