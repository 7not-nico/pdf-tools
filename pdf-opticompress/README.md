# pdf-opticompress

A command-line tool to optimize PDF files by compressing images and reducing file size.

## Features

- Optimize single PDF files with customizable quality and presets
- Analyze PDF structure and estimate potential savings
- Batch process multiple PDFs in parallel using multiple threads

## Installation

Ensure Rust is installed, then:

```bash
cargo build --release
```

## Usage

### Optimize a PDF

```bash
./target/release/pdf-opticompress optimize input.pdf output.pdf --quality 80 --preset web
```

Options:
- `--quality`: Image quality (0-100, default 80)
- `--preset`: Optimization preset (web, print, max)

### Analyze a PDF

```bash
./target/release/pdf-opticompress analyze input.pdf --show-savings
```

Shows file structure, image count, and potential compression savings.

### Batch process

```bash
./target/release/pdf-opticompress batch file1.pdf file2.pdf --output-dir optimized/ --threads 4
```

Processes multiple files in parallel.