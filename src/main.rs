use clap::Parser;
use deepscene::cli;
use deepscene::processor::{DataProcessor, DecodeResult, EncodeResult};

fn print_encode_result(result: &EncodeResult) {
    println!(
        "File hidden successfully in '{}'",
        result.output_path.display()
    );

    println!("File: {}", result.file_name);
    println!("Encrypted: {}", if result.encrypted { "Yes" } else { "No" });
    println!(
        "Compressed: {}",
        if result.compressed { "Yes" } else { "No" }
    );

    if result.converted_to_png {
        println!("Converted to PNG: Yes");
    }

    let percentage = if result.final_size < result.original_size {
        ((result.original_size - result.final_size) as f64 / result.original_size as f64) * 100.0
    } else {
        0.0
    };

    if result.compressed {
        println!(
            "Original size: {} bytes, Final size: {} bytes ({:.2}% reduction)",
            result.original_size, result.final_size, percentage
        );
    } else {
        println!("Payload size: {} bytes", result.original_size);
    }

    println!("\nNOTE:\nOnly lossless formats (PNG, BMP, TIFF) preserve hidden data.");
    println!("Lossy formats (JPEG, WebP) will corrupt the embedded information.\n");
}

fn print_decode_result(result: &DecodeResult) {
    println!(
        "File extracted successfully to '{}'",
        result.output_path.display()
    );
    println!("File name: {}", result.file_name);
    println!("Encrypted: {}", if result.encrypted { "Yes" } else { "No" });
    println!("Extracted {} bytes\n", result.file_size);
}

fn handle_encode(
    input: std::path::PathBuf,
    file: std::path::PathBuf,
    output: Option<std::path::PathBuf>,
    password: Option<String>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let options = deepscene::processor::EncodeOptions {
        file_path: file,
        image_path: input,
        output_path: output,
        password,
    };

    let result = DataProcessor::encode(options)?;
    print_encode_result(&result);

    Ok(())
}

fn handle_decode(
    input: std::path::PathBuf,
    output: Option<std::path::PathBuf>,
    password: Option<String>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let options = deepscene::processor::DecodeOptions {
        image_path: input,
        output_path: output,
        password,
    };

    let result = DataProcessor::decode(options)?;
    print_decode_result(&result);

    Ok(())
}

fn main() {
    let cli = cli::Cli::parse();

    let result = match cli.command {
        cli::Commands::Encode {
            input,
            file,
            output,
            password,
        } => handle_encode(input, file, output, password),
        cli::Commands::Decode {
            input,
            output,
            password,
        } => handle_decode(input, output, password),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
