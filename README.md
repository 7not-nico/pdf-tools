# PDF Tools

A collection of PDF utilities written in Rust.

## Projects

### pdf-opticompress
Optimizes PDF files by compressing images and other elements.

### pdf-renamer
Renames PDF files based on their content or metadata.

## Installation

Ensure you have Rust installed. Then, for each project:

```bash
cd pdf-opticompress
cargo build --release

cd ../pdf-renamer
cargo build --release
```

## Usage

Run the binaries from their respective directories.

For pdf-opticompress:
```bash
./target/release/pdf-opticompress --help
```

For pdf-renamer:
```bash
./target/release/pdf-renamer --help
```