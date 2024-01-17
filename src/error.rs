use rodio::decoder::DecoderError;
use rodio::StreamError;
use std::io;

pub type AppResult<T> = Result<T, AppError>;

/// Custom application error type
#[derive(Debug, Clone)]
pub struct CustomAppError {
    pub message: String,
}

impl std::fmt::Display for CustomAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
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

impl AppError {
    pub fn new(message: &str) -> Self {
        Self::Custom(CustomAppError {
            message: message.to_string(),
        })
    }
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
            Self::Custom(error) => write!(f, "Error: {}", error),
        }
    }
}
