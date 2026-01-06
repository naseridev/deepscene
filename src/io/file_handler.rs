use crate::core::error::{DeepSceneError, Result};
use std::fs;
use std::path::Path;

const MAX_FILE_SIZE: usize = 256 * 1024 * 1024;
const MAX_FILENAME_LENGTH: usize = 255;

pub struct FileData {
    pub name: String,
    pub data: Vec<u8>,
}

pub struct FileHandler;

impl FileHandler {
    pub fn read_file(path: &Path) -> Result<FileData> {
        if !path.exists() {
            return Err(DeepSceneError::Validation(format!(
                "File '{}' not found",
                path.display()
            )));
        }

        if !path.is_file() {
            return Err(DeepSceneError::Validation(format!(
                "'{}' is not a file",
                path.display()
            )));
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| DeepSceneError::Validation("Invalid file name".to_string()))?
            .to_string();

        if file_name.is_empty() {
            return Err(DeepSceneError::Validation(
                "File name cannot be empty".to_string(),
            ));
        }

        if file_name.len() > MAX_FILENAME_LENGTH {
            return Err(DeepSceneError::Validation(format!(
                "File name too long (max {} bytes)",
                MAX_FILENAME_LENGTH
            )));
        }

        let data = fs::read(path)?;

        if data.is_empty() {
            return Err(DeepSceneError::Validation(format!(
                "File '{}' is empty",
                path.display()
            )));
        }

        if data.len() > MAX_FILE_SIZE {
            return Err(DeepSceneError::Validation(format!(
                "File '{}' is too large. Maximum file size is {} MB, but file is {} MB",
                path.display(),
                MAX_FILE_SIZE / (1024 * 1024),
                data.len() / (1024 * 1024)
            )));
        }

        Ok(FileData {
            name: file_name,
            data,
        })
    }

    pub fn write_file(path: &Path, data: &[u8]) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(DeepSceneError::Validation(format!(
                    "Output directory '{}' does not exist",
                    parent.display()
                )));
            }
        }

        if path.exists() && !path.is_file() {
            return Err(DeepSceneError::Validation(format!(
                "Output path '{}' exists but is not a file",
                path.display()
            )));
        }

        fs::write(path, data)?;
        Ok(())
    }

    pub fn validate_output_path(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(DeepSceneError::Validation(format!(
                    "Output directory '{}' does not exist",
                    parent.display()
                )));
            }
        }

        if path.exists() && !path.is_file() {
            return Err(DeepSceneError::Validation(format!(
                "Output path '{}' exists but is not a file",
                path.display()
            )));
        }

        Ok(())
    }
}
