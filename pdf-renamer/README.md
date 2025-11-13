# pdf-renamer

A command-line tool to rename PDF files based on their metadata or content.

## Features

- Rename single PDF files using title, author, or content
- Batch rename all PDFs in a directory
- Extract metadata from PDF info dictionary
- Fallback to text extraction from first page if metadata is missing
- Sanitize filenames for filesystem compatibility

## Installation

Ensure Rust is installed, then:

```bash
cargo build --release
```

## Usage

### Rename a single PDF

```bash
./target/release/pdf-renamer --input file.pdf --pattern title
```

Renames based on PDF title metadata.

### Batch rename in directory

```bash
./target/release/pdf-renamer --input /path/to/dir --pattern title
```

Renames all PDFs in the directory.

Options:
- `--pattern`: 'title' (default) or 'filename' (keep original)