use rodio::decoder::DecoderError;
use rodio::StreamError;
use std::io;

pub type AppResult<T> = Result<T, AppError>;

// /// Define custom error types
// #[derive(Debug, Clone, PartialEq)]
// pub enum CustomError {
//     Interrupted,
// }

/// Custom application error type
#[derive(Debug, Clone)]
pub struct CustomAppError {
    pub message: String,
}

impl CustomAppError {
    #[allow(dead_code)]
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::error::Error for CustomAppError {
    fn description(&self) -> &str {
        "sending on a closed channel"
    }
}

impl std::fmt::Display for CustomAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

/// Application error type
#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Decoder(DecoderError),
    Stream(StreamError),
    Serde(serde_json::Error),
    Notify(notify_rust::error::Error),
    Custom(CustomAppError),
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<DecoderError> for AppError {
    fn from(error: DecoderError) -> Self {
        Self::Decoder(error)
    }
}

impl From<StreamError> for AppError {
    fn from(error: StreamError) -> Self {
        Self::Stream(error)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serde(error)
    }
}

impl From<notify_rust::error::Error> for AppError {
    fn from(error: notify_rust::error::Error) -> Self {
        Self::Notify(error)
    }
}

impl From<CustomAppError> for AppError {
    fn from(error: CustomAppError) -> Self {
        Self::Custom(error)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "IO Error: {}", error),
            Self::Decoder(error) => write!(f, "Decoder Error: {}", error),
            Self::Stream(error) => write!(f, "Stream Error: {}", error),
            Self::Serde(error) => write!(f, "Serde Error: {}", error),
            Self::Notify(error) => write!(f, "Notify Error: {}", error),
            Self::Custom(error) => write!(f, "Custom Error: {}", error),
        }
    }
}

/// Implement std::error::Error for AppError
impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Decoder(error) => Some(error),
            Self::Stream(error) => Some(error),
            Self::Serde(error) => Some(error),
            Self::Notify(error) => Some(error),
            Self::Custom(error) => Some(error),
        }
    }
}
