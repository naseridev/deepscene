use std::fmt;

#[derive(Debug)]
pub enum DeepSceneError {
    Io(std::io::Error),
    Image(String),
    Encryption(String),
    Compression(String),
    Validation(String),
    Data(String),
}

impl fmt::Display for DeepSceneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeepSceneError::Io(e) => write!(f, "IO error: {}", e),
            DeepSceneError::Image(e) => write!(f, "Image error: {}", e),
            DeepSceneError::Encryption(e) => write!(f, "Encryption error: {}", e),
            DeepSceneError::Compression(e) => write!(f, "Compression error: {}", e),
            DeepSceneError::Validation(e) => write!(f, "Validation error: {}", e),
            DeepSceneError::Data(e) => write!(f, "Data error: {}", e),
        }
    }
}

impl std::error::Error for DeepSceneError {}

impl From<std::io::Error> for DeepSceneError {
    fn from(err: std::io::Error) -> Self {
        DeepSceneError::Io(err)
    }
}

impl From<image::ImageError> for DeepSceneError {
    fn from(err: image::ImageError) -> Self {
        DeepSceneError::Image(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DeepSceneError>;
