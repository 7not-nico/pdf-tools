# PDF Tools

A collection of PDF utilities written in Rust.

## Projects

### pdf-opticompress
Optimizes PDF files by compressing images and other elements.

### pdf-renamer
Renames PDF files based on their metadata.

## Installation

### Quick Run (Easiest)
Run tools temporarily in your terminal:
```bash
curl -fsSL https://raw.githubusercontent.com/7not-nico/pdf-tools/main/run_online.sh | bash
```
This downloads the latest binary for your OS, runs it interactively, and cleans up.

### GitHub Actions
Use GitHub Actions to run tools online:
1. Go to [Actions Tab](https://github.com/7not-nico/pdf-tools/actions)
2. Select "Run PDF Tools" workflow
3. Click "Run workflow", fill in the PDF URL and options
4. View results in the workflow logs

### Online Interactive
Use GitHub Codespaces for full interactive experience:
1. Go to [GitHub Repo](https://github.com/7not-nico/pdf-tools)
2. Click "Code" > "Codespaces" > "Create codespace on main"
3. Wait for setup, then run `./pdf-opticompress` or `./pdf-renamer` for prompts

### From Source
Ensure you have Rust installed. Then, for each project:

```bash
cd pdf-opticompress
cargo build --release

cd ../pdf-renamer
cargo build --release
```

### Pre-built Binaries
Download the latest release binaries from [GitHub Releases](https://github.com/7not-nico/pdf-tools/releases) for Linux, Windows, and macOS. Make the binary executable and run from command line.

## Usage

Run the binaries from their respective directories or downloaded location.

For pdf-opticompress:
```bash
pdf-opticompress --help
```

For pdf-renamer:
```bash
pdf-renamer --help
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