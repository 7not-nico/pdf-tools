use anyhow::{Context, Result};
use lopdf::Document;
use std::path::Path;

/// Save options for PDF optimization
#[derive(Clone)]
pub struct SaveOptions {
    pub enable_compression: bool,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            enable_compression: true,
        }
    }
}

/// Save a PDF document with optimization options
pub fn save_pdf(doc: &mut Document, path: &Path, options: &SaveOptions) -> Result<()> {
    // Apply compression if enabled
    if options.enable_compression {
        doc.compress();
    }

    let _file = doc.save(path)
        .with_context(|| format!("Failed to save PDF: {}", path.display()))?;
    Ok(())
}

/// Create optimized save options based on preset
pub fn create_save_options_for_preset(preset: &crate::cli::Preset) -> SaveOptions {
    match preset {
        crate::cli::Preset::Web => SaveOptions {
            enable_compression: true,
        },
        crate::cli::Preset::Print => SaveOptions {
            enable_compression: true,
        },
        crate::cli::Preset::Archive => SaveOptions {
            enable_compression: true,
        },
        crate::cli::Preset::Maximum => SaveOptions {
            enable_compression: true,
        },
    }
}