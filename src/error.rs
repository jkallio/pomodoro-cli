use rodio::decoder::DecoderError;
use rodio::stream::DeviceSinkError;
use std::io;

pub type AppResult<T> = Result<T, AppError>;

/// Application error type
#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Decoder(DecoderError),
    DeviceSink(DeviceSinkError),
    Serde(serde_json::Error),
    Notify(notify_rust::error::Error),
    Custom(String),
}

impl AppError {
    pub fn new(message: &str) -> Self {
        Self::Custom(message.to_string())
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

impl From<DeviceSinkError> for AppError {
    fn from(error: DeviceSinkError) -> Self {
        Self::DeviceSink(error)
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

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "IO Error: {}", error),
            Self::Decoder(error) => write!(f, "Decoder Error: {}", error),
            Self::DeviceSink(error) => write!(f, "Device Sink Error: {}", error),
            Self::Serde(error) => write!(f, "Serde Error: {}", error),
            Self::Notify(error) => write!(f, "Notify Error: {}", error),
            Self::Custom(message) => write!(f, "Error: {}", message),
        }
    }
}
