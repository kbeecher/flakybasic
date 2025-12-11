use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum BasicError {
    SyntaxError(String),
    RuntimeError(String),
}

impl Display for BasicError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            BasicError::SyntaxError(msg) => write!(f, "{}", msg),
            BasicError::RuntimeError(msg) => write!(f, "{}", msg),
        }
    }
}
