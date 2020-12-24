use std::error::Error;
use std::fmt::{Display, Formatter, Result, Debug};

#[derive(Debug)]
pub struct LocalError {
    description : String,
}

impl LocalError {
    pub fn new(description: &str) -> LocalError {
        LocalError {
            description: description.into()
        }
    }
}

impl Error for LocalError {

}

impl Display for LocalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.description)
    }
}
