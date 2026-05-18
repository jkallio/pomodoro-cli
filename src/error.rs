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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_error_display() {
        let err = AppError::new("Test error message");
        assert_eq!(err.to_string(), "Error: Test error message");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert!(app_err.to_string().contains("IO Error"));
        assert!(app_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_serde_error_conversion() {
        let json_str = "{ invalid json }";
        let serde_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let app_err: AppError = serde_err.into();
        assert!(app_err.to_string().contains("Serde Error"));
    }

    #[test]
    fn test_custom_error_creation() {
        let err = AppError::Custom("custom message".to_string());
        match err {
            AppError::Custom(msg) => assert_eq!(msg, "custom message"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_io_error_variants() {
        let err1 = AppError::Io(io::Error::new(io::ErrorKind::PermissionDenied, "denied"));
        assert!(err1.to_string().contains("IO Error"));
        assert!(err1.to_string().contains("denied"));

        let err2 = AppError::Io(io::Error::new(io::ErrorKind::AlreadyExists, "exists"));
        assert!(err2.to_string().contains("IO Error"));
        assert!(err2.to_string().contains("exists"));
    }
}
