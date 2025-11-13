use std::fs;
use std::path::Path;

/// Check if a file exists and is readable
pub fn validate_input_file(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", path.display()),
        ));
    }

    if !path.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Not a file: {}", path.display()),
        ));
    }

    // Try to open the file to check readability
    fs::File::open(path)?;
    Ok(())
}

/// Get file size in bytes
pub fn get_file_size(path: &Path) -> std::io::Result<u64> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Calculate compression ratio
pub fn calculate_compression_ratio(original: u64, compressed: u64) -> f64 {
    if original == 0 {
        return 0.0;
    }
    ((original as f64 - compressed as f64) / original as f64) * 100.0
}