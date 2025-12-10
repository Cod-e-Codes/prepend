//! Custom error types for the prepend tool.

use std::fmt;
use std::io;

/// Error type for prepend operations
#[derive(Debug)]
pub enum PrependError {
    /// File does not exist
    FileNotFound(String),

    /// Path is not a regular file
    NotAFile(String),

    /// File is not writable
    NotWritable(String),

    /// Input text is empty
    EmptyInput,

    /// I/O error occurred
    Io(io::Error),
}

impl fmt::Display for PrependError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrependError::FileNotFound(path) => write!(f, "File {} does not exist.", path),
            PrependError::NotAFile(path) => write!(f, "{} is not a regular file.", path),
            PrependError::NotWritable(path) => write!(f, "File {} is not writable.", path),
            PrependError::EmptyInput => write!(f, "Input text is empty."),
            PrependError::Io(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for PrependError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PrependError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for PrependError {
    fn from(err: io::Error) -> Self {
        PrependError::Io(err)
    }
}
