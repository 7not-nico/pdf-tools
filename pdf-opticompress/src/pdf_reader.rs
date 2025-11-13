use anyhow::{Context, Result};
use lopdf::Document;
use std::path::Path;

/// Load a PDF document from file
pub fn load_pdf(path: &Path) -> Result<Document> {
    Document::load(path)
        .with_context(|| format!("Failed to load PDF: {}", path.display()))
}

/// Validate that the loaded document is valid
pub fn validate_pdf(doc: &Document) -> Result<()> {
    // Basic validation - check if document has pages
    if doc.get_pages().is_empty() {
        return Err(anyhow::anyhow!("PDF document contains no pages"));
    }

    // Check if document has a root catalog
    if let Err(_) = doc.trailer.get(b"Root") {
        return Err(anyhow::anyhow!("PDF document is missing root catalog"));
    }

    Ok(())
}

/// Get basic document information
pub struct PdfInfo {
    pub page_count: usize,
    pub version: String,
    pub has_encryption: bool,
}

pub fn get_pdf_info(doc: &Document) -> PdfInfo {
    let page_count = doc.get_pages().len();
    let version = doc.version.clone();
    let has_encryption = doc.is_encrypted();

    PdfInfo {
        page_count,
        version,
        has_encryption,
    }
}