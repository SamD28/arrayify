use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ArrayifyError {
    CommandError(String),
    FarmError(String),
}

impl Display for ArrayifyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrayifyError::CommandError(msg) => write!(f, "Command error: {}", msg),
            ArrayifyError::FarmError(msg) => write!(f, "Farm error: {}", msg),
        }
    }
}

impl std::error::Error for ArrayifyError {}

impl From<std::io::Error> for ArrayifyError {
    fn from(e: std::io::Error) -> Self {
        ArrayifyError::FarmError(e.to_string())
    }
}