use crate::core::{CompressionEngine, CryptoEngine, Result, SteganographyEngine};
use crate::io::FileHandler;
use std::path::PathBuf;

#[derive(Debug)]
pub struct EncodeOptions {
    pub file_path: PathBuf,
    pub image_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub password: Option<String>,
}

#[derive(Debug)]
pub struct DecodeOptions {
    pub image_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub password: Option<String>,
}

#[derive(Debug)]
pub struct EncodeResult {
    pub output_path: PathBuf,
    pub file_name: String,
    pub original_size: usize,
    pub final_size: usize,
    pub encrypted: bool,
    pub compressed: bool,
    pub converted_to_png: bool,
}

#[derive(Debug)]
pub struct DecodeResult {
    pub output_path: PathBuf,
    pub file_name: String,
    pub file_size: usize,
    pub encrypted: bool,
}

pub struct DataProcessor;

impl DataProcessor {
    pub fn encode(options: EncodeOptions) -> Result<EncodeResult> {
        println!("> DeepScene is here \n");

        let mut working_image_path = options.image_path.clone();
        let mut converted_to_png = false;

        if !SteganographyEngine::is_lossless_format(&options.image_path) {
            println!("[1/6] Converting image to lossless format (PNG)...");
            working_image_path = SteganographyEngine::convert_to_lossless(&options.image_path)?;
            converted_to_png = true;
            println!("      > Converted to PNG format");
        }

        let step_offset = if converted_to_png { 1 } else { 0 };

        println!("[{}/{}] Reading file...", 1 + step_offset, 5 + step_offset);
        let file_data = FileHandler::read_file(&options.file_path)?;

        println!(
            "      > File read successfully: {} bytes",
            file_data.data.len()
        );
        println!(
            "[{}/{}] Preparing payload...",
            2 + step_offset,
            5 + step_offset
        );

        let mut payload = Vec::new();
        let name_len = file_data.name.len() as u8;

        payload.push(name_len);
        payload.extend_from_slice(file_data.name.as_bytes());

        let encryption_flag = if options.password.is_some() { 1u8 } else { 0u8 };
        payload.push(encryption_flag);

        let file_data_to_store = if let Some(ref pwd) = options.password {
            CryptoEngine::encrypt(&file_data.data, pwd)?
        } else {
            file_data.data.clone()
        };

        payload.extend_from_slice(&file_data_to_store);

        println!("      > Payload prepared");
        println!(
            "[{}/{}] Analyzing and compressing data...",
            3 + step_offset,
            5 + step_offset
        );

        let original_payload_size = payload.len();
        let (processed_data, compression_applied) = CompressionEngine::compress(&payload)?;

        let compression_flag = if compression_applied { 1u8 } else { 0u8 };
        let mut final_payload = vec![compression_flag];
        final_payload.extend_from_slice(&processed_data);

        let final_size = final_payload.len();

        if compression_applied {
            let reduction = ((original_payload_size - processed_data.len()) as f64
                / original_payload_size as f64)
                * 100.0;
            println!(
                "      > Compression applied: {} bytes -> {} bytes ({:.2}% reduction)",
                original_payload_size,
                processed_data.len(),
                reduction
            );
        } else {
            println!(
                "      > Compression skipped: would not reduce size ({} bytes)",
                original_payload_size
            );
        }

        println!(
            "[{}/{}] Validating output path...",
            4 + step_offset,
            5 + step_offset
        );
        let output_path = options.output_path.unwrap_or_else(|| {
            let mut path = options.image_path.clone();
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            path.set_file_name(format!("{}_steg.png", stem));
            path
        });

        FileHandler::validate_output_path(&output_path)?;

        println!("      > Output path validated");
        println!(
            "[{}/{}] Embedding data into image...",
            5 + step_offset,
            5 + step_offset
        );

        SteganographyEngine::hide_data(&working_image_path, &final_payload, &output_path)?;

        println!("      > Data embedded successfully \n");
        println!("> Encoding complete \n");

        Ok(EncodeResult {
            output_path,
            file_name: file_data.name,
            original_size: original_payload_size,
            final_size,
            encrypted: options.password.is_some(),
            compressed: compression_applied,
            converted_to_png,
        })
    }

    pub fn decode(options: DecodeOptions) -> Result<DecodeResult> {
        println!("> DeepScene is here \n");

        println!("[1/4] Extracting data from image...");

        let embedded_data = SteganographyEngine::extract_data(&options.image_path)?;
        println!("      > Extracted {} bytes", embedded_data.len());

        if embedded_data.is_empty() {
            return Err(crate::core::DeepSceneError::Data(
                "No data found in image".to_string(),
            ));
        }

        let compression_flag = embedded_data[0];
        let payload_data = &embedded_data[1..];

        println!("[2/4] Processing data...");

        let decompressed_data = if compression_flag == 1 {
            let decompressed = CompressionEngine::decompress(payload_data)?;
            println!(
                "      > Decompressed: {} bytes -> {} bytes",
                payload_data.len(),
                decompressed.len()
            );
            decompressed
        } else {
            println!("      > No compression detected");
            payload_data.to_vec()
        };

        if decompressed_data.is_empty() {
            return Err(crate::core::DeepSceneError::Data(
                "Processed data is empty".to_string(),
            ));
        }

        println!("[3/4] Parsing metadata...");

        let name_len = decompressed_data[0] as usize;

        if name_len == 0 {
            return Err(crate::core::DeepSceneError::Data(
                "Invalid file name length (0)".to_string(),
            ));
        }

        if name_len > 255 {
            return Err(crate::core::DeepSceneError::Data(format!(
                "Invalid file name length ({}). Maximum is 255",
                name_len
            )));
        }

        if decompressed_data.len() < 1 + name_len + 1 {
            return Err(crate::core::DeepSceneError::Data(
                "Invalid data structure: missing encryption flag".to_string(),
            ));
        }

        let file_name =
            String::from_utf8(decompressed_data[1..1 + name_len].to_vec()).map_err(|e| {
                crate::core::DeepSceneError::Data(format!("Failed to decode file name: {}", e))
            })?;

        if file_name.is_empty() {
            return Err(crate::core::DeepSceneError::Data(
                "File name is empty".to_string(),
            ));
        }

        if file_name.contains('\0') {
            return Err(crate::core::DeepSceneError::Data(
                "File name contains null bytes".to_string(),
            ));
        }

        let encryption_flag = decompressed_data[1 + name_len];
        let encrypted_data = &decompressed_data[1 + name_len + 1..];

        let file_data = if encryption_flag == 1 {
            match options.password {
                Some(ref pwd) => CryptoEngine::decrypt(encrypted_data, pwd)?,
                None => {
                    return Err(crate::core::DeepSceneError::Validation(
                        "File is password-protected. Please provide the decryption password using -p or --password flag".to_string()
                    ));
                }
            }
        } else {
            if options.password.is_some() {
                return Err(crate::core::DeepSceneError::Validation(
                    "Password provided for unencrypted file. This file does not require a password"
                        .to_string(),
                ));
            }
            encrypted_data.to_vec()
        };

        if file_data.is_empty() {
            return Err(crate::core::DeepSceneError::Data(
                "Extracted file data is empty".to_string(),
            ));
        }

        println!("      > Metadata parsed successfully");
        println!("[4/4] Writing output file...");

        let output_path = options
            .output_path
            .unwrap_or_else(|| PathBuf::from(&file_name));

        FileHandler::write_file(&output_path, &file_data)?;

        println!("      > File written: {} bytes \n", file_data.len());
        println!("> Decoding complete \n");

        Ok(DecodeResult {
            output_path,
            file_name,
            file_size: file_data.len(),
            encrypted: encryption_flag == 1,
        })
    }
}
