use std::error::Error;
use std::fmt::{Display, Formatter, Result, Debug};

#[derive(Debug)]
pub struct TaskError {
    description : String,
}

impl TaskError {
    pub fn new(description: String) -> Self {
        Self {
            description: description
        }
    }
}

impl Error for TaskError {

}

impl Display for TaskError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.description)
    }
}
