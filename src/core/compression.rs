use crate::core::error::{DeepSceneError, Result};
use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use std::io::{Read, Write};

pub struct CompressionEngine;

impl CompressionEngine {
    pub fn compress(data: &[u8]) -> Result<(Vec<u8>, bool)> {
        let original_size = data.len();

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(data)
            .map_err(|e| DeepSceneError::Compression(format!("Failed to compress data: {}", e)))?;

        let compressed = encoder.finish().map_err(|e| {
            DeepSceneError::Compression(format!("Failed to finalize compression: {}", e))
        })?;

        let compressed_size = compressed.len();

        let threshold = (original_size as f64 * 0.95) as usize;

        if compressed_size < threshold {
            Ok((compressed, true))
        } else {
            Ok((data.to_vec(), false))
        }
    }

    pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = DeflateDecoder::new(data);
        let mut result = Vec::new();

        decoder.read_to_end(&mut result).map_err(|e| {
            DeepSceneError::Compression(format!("Failed to decompress data: {}", e))
        })?;

        Ok(result)
    }
}
