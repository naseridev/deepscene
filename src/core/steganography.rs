use crate::core::error::{DeepSceneError, Result};
use image::{GenericImageView, RgbaImage};
use std::path::{Path, PathBuf};

const MAX_IMAGE_DIMENSION: u32 = 20000;
const MAX_DATA_LENGTH: usize = 256 * 1024 * 1024;
const HEADER_MAGIC: &[u8; 4] = b"DPSN";

pub struct SteganographyEngine;

impl SteganographyEngine {
    pub fn is_lossless_format(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                let ext_lower = ext_str.to_lowercase();
                return matches!(ext_lower.as_str(), "png" | "bmp" | "tiff" | "tif");
            }
        }
        false
    }

    pub fn convert_to_lossless(image_path: &Path) -> Result<PathBuf> {
        let img = image::open(image_path).map_err(|e| {
            DeepSceneError::Image(format!(
                "Failed to open image '{}': {}",
                image_path.display(),
                e
            ))
        })?;

        let temp_path = image_path.with_extension("png");

        img.save(&temp_path)
            .map_err(|e| DeepSceneError::Image(format!("Failed to convert image to PNG: {}", e)))?;

        Ok(temp_path)
    }

    pub fn validate_image(path: &Path) -> Result<(u32, u32)> {
        if !path.exists() {
            return Err(DeepSceneError::Validation(format!(
                "Input image '{}' not found",
                path.display()
            )));
        }

        if !path.is_file() {
            return Err(DeepSceneError::Validation(format!(
                "'{}' is not a file",
                path.display()
            )));
        }

        let img = image::open(path).map_err(|e| {
            DeepSceneError::Image(format!("Failed to open image '{}': {}", path.display(), e))
        })?;

        let (width, height) = img.dimensions();

        if width == 0 || height == 0 {
            return Err(DeepSceneError::Validation(
                "Image has invalid dimensions".to_string(),
            ));
        }

        if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
            return Err(DeepSceneError::Validation(format!(
                "Image dimensions too large ({}x{}). Maximum is {}x{} pixels",
                width, height, MAX_IMAGE_DIMENSION, MAX_IMAGE_DIMENSION
            )));
        }

        Ok((width, height))
    }

    pub fn calculate_capacity(width: u32, height: u32) -> usize {
        let total_pixels = width as u64 * height as u64;
        ((total_pixels * 3) / 8) as usize
    }

    fn calculate_header_checksum(data: &[u8]) -> u16 {
        data.iter().fold(0u16, |acc, &b| acc.wrapping_add(b as u16))
    }

    pub fn hide_data(image_path: &Path, data: &[u8], output_path: &Path) -> Result<()> {
        let (width, height) = Self::validate_image(image_path)?;

        let img = image::open(image_path)?;
        let mut rgba_img = img.to_rgba8();

        let max_bytes = Self::calculate_capacity(width, height);
        let required_bytes = data.len() + 10;

        if required_bytes > max_bytes {
            let max_data_size = max_bytes.saturating_sub(10);
            let min_pixels_needed = ((required_bytes * 8) as f64 / 3.0).ceil() as u64;
            let min_dimension = (min_pixels_needed as f64).sqrt().ceil() as u32;

            return Err(DeepSceneError::Validation(format!(
                "Data too large for image. Image can hold {} bytes, but {} bytes needed. Try using an image at least {}x{} pixels.",
                max_data_size,
                data.len(),
                min_dimension,
                min_dimension
            )));
        }

        Self::embed_data(&mut rgba_img, data)?;

        rgba_img.save(output_path).map_err(|e| {
            DeepSceneError::Image(format!(
                "Failed to save output image '{}': {}",
                output_path.display(),
                e
            ))
        })?;

        Ok(())
    }

    fn embed_data(image: &mut RgbaImage, data: &[u8]) -> Result<()> {
        let length = data.len() as u32;
        let length_bytes = length.to_be_bytes();

        let mut header = Vec::new();
        header.extend_from_slice(HEADER_MAGIC);
        header.extend_from_slice(&length_bytes);

        let checksum = Self::calculate_header_checksum(&header);
        header.extend_from_slice(&checksum.to_be_bytes());

        let mut all_data = Vec::new();
        all_data.extend_from_slice(&header);
        all_data.extend_from_slice(data);

        let (width, height) = image.dimensions();
        let mut bit_index = 0;

        'outer: for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel_mut(x, y);

                for channel in 0..3 {
                    if bit_index >= all_data.len() * 8 {
                        break 'outer;
                    }

                    let byte_index = bit_index / 8;
                    let bit_position = 7 - (bit_index % 8);
                    let bit = (all_data[byte_index] >> bit_position) & 1;

                    pixel[channel] = (pixel[channel] & 0xFE) | bit;
                    bit_index += 1;
                }
            }
        }

        Ok(())
    }

    pub fn extract_data(image_path: &Path) -> Result<Vec<u8>> {
        let (width, height) = Self::validate_image(image_path)?;

        let img = image::open(image_path)?;
        let rgba_img = img.to_rgba8();

        let total_pixels = width as usize * height as usize;
        let max_bits = (total_pixels * 3).min(80 + (MAX_DATA_LENGTH * 8));

        let mut all_bits = Vec::with_capacity(max_bits.min(100000));

        'outer: for y in 0..height {
            for x in 0..width {
                let pixel = rgba_img.get_pixel(x, y);

                for channel in 0..3 {
                    all_bits.push(pixel[channel] & 1);
                    if all_bits.len() >= max_bits {
                        break 'outer;
                    }
                }
            }
        }

        if all_bits.len() < 80 {
            return Err(DeepSceneError::Data(
                "Image dimensions insufficient for data extraction".to_string(),
            ));
        }

        Self::validate_and_extract(&all_bits, width, height)
    }

    fn validate_and_extract(bits: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
        let mut header = Vec::new();
        for i in 0..10 {
            let mut byte = 0u8;
            for j in 0..8 {
                byte = (byte << 1) | bits[i * 8 + j];
            }
            header.push(byte);
        }

        if &header[0..4] != HEADER_MAGIC {
            return Err(DeepSceneError::Data(
                "No embedded data detected. This image does not appear to contain steganographic content".to_string()
            ));
        }

        let stored_checksum = u16::from_be_bytes([header[8], header[9]]);
        let computed_checksum = Self::calculate_header_checksum(&header[0..8]);

        if stored_checksum != computed_checksum {
            return Err(DeepSceneError::Data(
                "Data integrity check failed. The embedded data may be corrupted".to_string(),
            ));
        }

        let data_length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;

        if data_length == 0 {
            return Err(DeepSceneError::Data(
                "No embedded data detected".to_string(),
            ));
        }

        if data_length > MAX_DATA_LENGTH {
            return Err(DeepSceneError::Data(format!(
                "Invalid data length detected ({} bytes). Maximum is {} MB.",
                data_length,
                MAX_DATA_LENGTH / (1024 * 1024)
            )));
        }

        let total_bits_needed = 80 + (data_length * 8);
        let available_bits = width as usize * height as usize * 3;

        if total_bits_needed > available_bits {
            return Err(DeepSceneError::Data(format!(
                "Image capacity exceeded. Required: {} bits. Available: {} bits",
                total_bits_needed, available_bits
            )));
        }

        if bits.len() < total_bits_needed {
            return Err(DeepSceneError::Data(format!(
                "Cannot extract data: need {} bits but only {} bits available",
                total_bits_needed,
                bits.len()
            )));
        }

        Self::extract_bytes(bits, data_length)
    }

    fn extract_bytes(bits: &[u8], length: usize) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(length);

        for i in 0..length {
            let mut byte = 0u8;
            for j in 0..8 {
                let bit_pos = 80 + (i * 8) + j;
                if bit_pos >= bits.len() {
                    return Err(DeepSceneError::Data(
                        "Unexpected end of data while extracting".to_string(),
                    ));
                }
                byte = (byte << 1) | bits[bit_pos];
            }
            data.push(byte);
        }

        Ok(data)
    }
}
