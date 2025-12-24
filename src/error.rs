use std::fmt;

#[derive(Debug)]
pub enum ScpError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Json(serde_json::Error),
    InvalidPath(String),
    InvalidFormat(String),
}

impl fmt::Display for ScpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScpError::Io(err) => write!(f, "IO error: {}", err),
            ScpError::Zip(err) => write!(f, "ZIP error: {}", err),
            ScpError::Json(err) => write!(f, "JSON error: {}", err),
            ScpError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            ScpError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl std::error::Error for ScpError {}

impl From<std::io::Error> for ScpError {
    fn from(err: std::io::Error) -> Self {
        ScpError::Io(err)
    }
}

impl From<zip::result::ZipError> for ScpError {
    fn from(err: zip::result::ZipError) -> Self {
        ScpError::Zip(err)
    }
}

impl From<serde_json::Error> for ScpError {
    fn from(err: serde_json::Error) -> Self {
        ScpError::Json(err)
    }
}

pub type Result<T> = std::result::Result<T, ScpError>;