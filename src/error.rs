use std::error::Error;
use std::fmt::{Display, Formatter, Result, Debug};

#[derive(Debug)]
pub struct InfcoError {
    description : String,
}

impl InfcoError {
    pub fn new(description: &str) -> InfcoError {
        InfcoError {
            description: description.into()
        }
    }
}

impl Error for InfcoError {

}

impl Display for InfcoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.description)
    }
}
