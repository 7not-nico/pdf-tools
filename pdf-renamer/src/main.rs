use clap::Parser;
use lopdf::{Document, Object};
use rayon::prelude::*;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use tempfile;

#[derive(Parser)]
#[command(name = "pdf-renamer")]
#[command(about = "Rename PDF files based on their metadata")]
struct Args {
    /// Path to the PDF file or directory containing PDFs
    #[arg(short, long)]
    input: Option<String>,

    /// Rename pattern: 'title' for title metadata, 'filename' to keep original
    #[arg(short, long, default_value = "title")]
    pattern: String,
}

fn main() {
    let mut args = Args::parse();

    if args.input.is_none() {
        print!("Enter path to PDF file or directory (URL or local): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        args.input = Some(input.trim().to_string());
        print!("Enter pattern (title or filename, default title): ");
        io::stdout().flush().unwrap();
        let mut pattern = String::new();
        io::stdin().read_line(&mut pattern).unwrap();
        if !pattern.trim().is_empty() {
            args.pattern = pattern.trim().to_string();
        }
    }

    let input = args.input.unwrap();
    let input_path = resolve_input_path(&input).unwrap();
    if Path::new(&input_path).is_dir() {
        // Batch rename
        println!("Batch renaming PDFs in directory: {}", input_path);
        batch_rename_pdfs(&input_path, &args.pattern);
    } else {
        // Single file
        rename_single_pdf(&input_path, &args.pattern);
    }
}

fn rename_single_pdf(path: &str, pattern: &str) {
    let doc = Document::load(path).expect("Failed to load PDF");
    let new_name = if pattern == "title" {
        let title = extract_title(&doc)
            .or_else(|| extract_concise_content(&doc))
            .unwrap_or_else(|| "Untitled".to_string());
        let author = extract_author(&doc);
        let base_name = if let Some(auth) = author {
            format!("{} - {}", title, auth)
        } else {
            title
        };
        let concise_name = make_concise_filename(&base_name);
        format!("{}.pdf", concise_name)
    } else {
        // For now, keep original
        Path::new(path).file_name().unwrap().to_string_lossy().to_string()
    };
    let new_path = Path::new(path).with_file_name(new_name);
    fs::rename(path, &new_path).expect("Failed to rename file");
    println!("Renamed {} to {}", path, new_path.display());
}

fn batch_rename_pdfs(dir: &str, pattern: &str) {
    let pdf_paths: Vec<String> = fs::read_dir(dir)
        .expect("Failed to read directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                Some(path.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();

    pdf_paths.par_iter().for_each(|path| {
        rename_single_pdf(path, pattern);
    });
}

fn extract_title(doc: &Document) -> Option<String> {
    let trailer = &doc.trailer;
    if let Ok(Object::Reference(info_ref)) = trailer.get(b"Info") {
        if let Ok(Object::Dictionary(info_dict)) = doc.get_object(*info_ref) {
            if let Ok(Object::String(title_bytes, _)) = info_dict.get(b"Title") {
                let title = String::from_utf8_lossy(&title_bytes).to_string();
                if !title.trim().is_empty() {
                    Some(title)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn extract_author(doc: &Document) -> Option<String> {
    let trailer = &doc.trailer;
    if let Ok(Object::Reference(info_ref)) = trailer.get(b"Info") {
        if let Ok(Object::Dictionary(info_dict)) = doc.get_object(*info_ref) {
            if let Ok(Object::String(author_bytes, _)) = info_dict.get(b"Author") {
                let author = String::from_utf8_lossy(&author_bytes).to_string();
                if !author.trim().is_empty() {
                    Some(author)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn extract_concise_content(doc: &Document) -> Option<String> {
    // Extract text from the first page
    let pages = doc.get_pages();
    if let Some(&page_id) = pages.keys().next() {
        if let Ok(text) = doc.extract_text(&[page_id]) {
            let content = text.trim();
            if !content.is_empty() {
                Some(content.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn make_concise_filename(name: &str) -> String {
    // Take first 100 chars, replace invalid filename chars with _, limit to 50
    let mut concise = name.chars().take(100).collect::<String>();
    concise = concise.replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-' && c != '_', "_");
    concise = concise.chars().take(50).collect();
    concise.trim().to_string()
}

fn resolve_input_path(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    if input.starts_with("http://") || input.starts_with("https://") {
        println!("Downloading from URL: {}", input);
        let response = reqwest::blocking::get(input)?;
        let temp_file = tempfile::NamedTempFile::new()?;
        let content = response.bytes()?;
        std::fs::write(temp_file.path(), content)?;
        Ok(temp_file.path().to_str().unwrap().to_string())
    } else {
        Ok(input.to_string())
    }
}
