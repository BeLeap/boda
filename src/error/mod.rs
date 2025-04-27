use std::{fmt, io};

pub type BodaResult<T> = Result<T, BodaError>;

pub enum BodaError {
    Io(io::Error),
    Custom(String),
}

impl fmt::Display for BodaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BodaError::Io(e) => write!(f, "Failed to handle io: {e}"),
            BodaError::Custom(e) => write!(f, "{}", e),
        }
    }
}

impl fmt::Debug for BodaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::convert::From<io::Error> for BodaError {
    fn from(value: io::Error) -> Self {
        BodaError::Io(value)
    }
}

// NOTE: implement detail later
impl std::convert::From<crossbeam_channel::SendError<bool>> for BodaError {
    fn from(value: crossbeam_channel::SendError<bool>) -> Self {
        BodaError::Custom(value.to_string())
    }
}
