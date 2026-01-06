use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "DeepScene")]
#[command(about = "A command-line steganography tool that embeds files into images with optional encryption and compression, then extracts them losslessly", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Embed a file into an image using steganography")]
    Encode {
        #[arg(help = "Path to the carrier image")]
        input: PathBuf,

        #[arg(help = "Path to the file to be embedded")]
        file: PathBuf,

        #[arg(
            short = 'o',
            long = "output",
            help = "Output path for the generated image (defaults to input_steg.png)"
        )]
        output: Option<PathBuf>,

        #[arg(
            short = 'p',
            long = "password",
            help = "Optional encryption password for securing the embedded data"
        )]
        password: Option<String>,
    },

    #[command(about = "Extract an embedded file from a steganographic image")]
    Decode {
        #[arg(help = "Path to the steganographic image")]
        input: PathBuf,

        #[arg(
            short = 'o',
            long = "output",
            help = "Output path for the extracted file (defaults to original filename)"
        )]
        output: Option<PathBuf>,

        #[arg(
            short = 'p',
            long = "password",
            help = "Decryption password if the embedded data was encrypted"
        )]
        password: Option<String>,
    },
}
