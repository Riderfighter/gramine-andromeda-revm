use std::error;
use std::fmt;

#[derive(Debug)]
pub(crate) struct FarcasterError {
    description: String,
}

impl FarcasterError {
    pub(crate) fn new(description: &str) -> Self {
        FarcasterError {
            description: description.to_string(),
        }
    }
}

impl fmt::Display for FarcasterError {
    pub(crate) fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl error::Error for FarcasterError {}