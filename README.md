# DeepScene

A command-line steganography tool that embeds files into images using LSB techniques with optional ChaCha20 encryption and DEFLATE compression.

## Overview

DeepScene hides arbitrary files within image carriers by manipulating the least significant bits of RGB channels. The tool preserves lossless image formats (PNG, BMP, TIFF) and automatically converts lossy formats during encoding. Embedded data includes a header with magic bytes, length field, and checksum for integrity verification.

## Installation

```bash
cargo build --release
```

The binary will be located at `target/release/deepscene`.

## Usage

### Encoding

Embed a file into an image:

```bash
deepscene encode <IMAGE> <FILE> [OPTIONS]
```

**Arguments:**
- `<IMAGE>` - Carrier image path
- `<FILE>` - File to embed

**Options:**
- `-o, --output <PATH>` - Output image path (default: `<input>_steg.png`)
- `-p, --password <PASSWORD>` - Encryption password

**Examples:**

```bash
deepscene encode carrier.png secret.txt
deepscene encode photo.jpg document.pdf -o hidden.png
deepscene encode image.png data.zip -p mypassword -o output.png
```

### Decoding

Extract an embedded file from an image:

```bash
deepscene decode <IMAGE> [OPTIONS]
```

**Arguments:**
- `<IMAGE>` - Steganographic image path

**Options:**
- `-o, --output <PATH>` - Output file path (default: original filename)
- `-p, --password <PASSWORD>` - Decryption password

**Examples:**

```bash
deepscene decode hidden.png
deepscene decode output.png -p mypassword
deepscene decode steg.png -o extracted.txt
```

## Technical Implementation

### Data Structure

The embedding format consists of:

1. **Compression Flag** (1 byte): `0x01` if DEFLATE applied, `0x00` otherwise
2. **Header** (10 bytes):
   - Magic bytes: `DPSN` (4 bytes)
   - Payload length: big-endian u32 (4 bytes)
   - Header checksum: big-endian u16 (2 bytes)
3. **Metadata**:
   - Filename length: u8 (1 byte)
   - Filename: UTF-8 string
   - Encryption flag: `0x01` if encrypted, `0x00` otherwise
4. **Payload**: File data (optionally encrypted)

### Processing Pipeline

**Encoding:**
1. Convert image to PNG if lossy format detected
2. Read input file and construct metadata
3. Apply ChaCha20 encryption if password provided
4. Compress payload using DEFLATE (skipped if size increases)
5. Embed data into LSB of RGB channels
6. Save output image

**Decoding:**
1. Extract LSB data from RGB channels
2. Validate header magic bytes and checksum
3. Decompress if compression flag set
4. Decrypt if encryption flag set and password provided
5. Parse metadata and extract filename
6. Write output file

### Cryptography

- **Key Derivation**: Argon2 with 16-byte random salt
- **Encryption**: ChaCha20 stream cipher with 12-byte random nonce
- **Integrity**: BLAKE3 hash (first 16 bytes) prepended to plaintext

### Capacity Calculation

Maximum embeddable bytes: `(width × height × 3) / 8`

For a 1920×1080 image: approximately 777,600 bytes (~760 KB).

## Limitations

### Size Constraints

- **Maximum file size**: 256 MB
- **Maximum image dimension**: 20,000 pixels per side
- **Maximum filename length**: 255 bytes

### Format Requirements

Lossy formats (JPEG, WebP) will corrupt embedded data. The tool automatically converts such inputs to PNG during encoding, but users must avoid re-saving output images in lossy formats.

### Security Considerations

- LSB steganography is detectable through statistical analysis
- No plausible deniability; header magic bytes identify embedded data
- Encryption uses password-based key derivation (vulnerable to weak passwords)
- No forward secrecy or authentication beyond BLAKE3 checksum

## Comparison with DeepSound

DeepScene is inspired by DeepSound, a Windows steganography tool for audio files. The name follows the same "Deep-" prefix convention, substituting "Scene" (image) for "Sound" (audio). Key differences:

### Architecture
- **DeepSound**: Closed-source Windows GUI application
- **DeepScene**: Open-source cross-platform CLI tool

### Medium
- **DeepSound**: Embeds data in audio files (WAV, FLAC)
- **DeepScene**: Embeds data in images (PNG, BMP, TIFF)

### Advantages over DeepSound
- **Portability**: Runs on Linux, macOS, Windows without dependencies
- **Automation**: CLI interface enables scripting and batch processing
- **Transparency**: Source code auditable for security verification
- **Modern Cryptography**: ChaCha20 cipher vs AES-256 (comparable security, faster on platforms without AES-NI)
- **Integrity Verification**: BLAKE3 checksums detect corruption during extraction
- **Adaptive Compression**: Automatically disables compression when counterproductive

### Trade-offs
- **Capacity**: Images typically hold less data per file size than audio
- **Usability**: CLI requires terminal proficiency vs GUI point-and-click
- **Detection Resistance**: Both LSB methods are equally vulnerable to statistical analysis
