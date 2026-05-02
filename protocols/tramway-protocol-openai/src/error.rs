use std::fmt;

#[derive(Debug)]
pub enum ProtocolError {
    /// The model string was empty.
    EmptyModel,
    /// The model string could not be parsed.
    InvalidModel(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::EmptyModel => write!(f, "model field must not be empty"),
            ProtocolError::InvalidModel(s) => write!(f, "invalid model string: '{s}'"),
        }
    }
}

impl std::error::Error for ProtocolError {}
