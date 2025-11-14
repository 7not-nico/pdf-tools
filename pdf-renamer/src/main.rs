use clap::Parser;
use lopdf::{Document, Object};
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::time::Instant;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "pdf-renamer")]
#[command(about = "Rename PDF files based on their metadata")]
struct Args {
    /// Path to the PDF file or directory containing PDFs
    #[arg(short, long)]
    input: String,

    /// Perform a dry run without actually renaming files
    #[arg(long)]
    dry_run: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let start = Instant::now();
    let args = Args::parse();

    let input_path = args.input;
    if Path::new(&input_path).is_dir() {
        // Batch rename
        println!("Batch renaming PDFs in directory: {}", input_path);
        batch_rename_pdfs(&input_path, args.dry_run, args.verbose);
    } else {
        // Single file
        rename_single_pdf(&input_path, args.dry_run, args.verbose);
    }

    let duration = start.elapsed();
    println!("Execution time: {:.2} seconds", duration.as_secs_f64());
}

fn rename_single_pdf(path: &str, dry_run: bool, verbose: bool) {
    let doc = Document::load(path).expect("Failed to load PDF");
    let title = extract_title(&doc)
        .or_else(|| extract_concise_content(&doc))
        .unwrap_or_else(|| "Untitled".to_string());
    if verbose {
        println!("Extracted title: '{}' for {}", &title, path);
    }
    let concise_name = make_concise_filename(&title);
    let new_name = format!("{}.pdf", concise_name);
    let new_path = Path::new(path).with_file_name(new_name);
    if dry_run {
        println!("Would rename {} to {}", path, new_path.display());
    } else {
        fs::rename(path, &new_path).expect("Failed to rename file");
        println!("Renamed {} to {}", path, new_path.display());
    }
}

fn batch_rename_pdfs(dir: &str, dry_run: bool, verbose: bool) {
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

    if verbose {
        println!("Found {} PDF files to process", pdf_paths.len());
    }

    // Compute proposed new names
    let mut proposed_renames: Vec<(String, String)> = pdf_paths
        .par_iter()
        .map(|path| {
            let doc = Document::load(path).expect("Failed to load PDF");
            let title = extract_title(&doc)
                .or_else(|| extract_concise_content(&doc))
                .unwrap_or_else(|| "Untitled".to_string());
            if verbose {
                println!("Extracted title: '{}' for {}", &title, path);
            }
            let concise_name = make_concise_filename(&title);
            let new_name = format!("{}.pdf", concise_name);
            (path.clone(), new_name)
        })
        .collect();

    // Handle duplicates
    let mut name_count: HashMap<String, usize> = HashMap::new();
    for (_, new_name) in &proposed_renames {
        *name_count.entry(new_name.clone()).or_insert(0) += 1;
    }

    let mut used_names: HashMap<String, usize> = HashMap::new();
    for (_path, new_name) in &mut proposed_renames {
        if name_count[&*new_name] > 1 {
            let count = used_names.entry(new_name.clone()).or_insert(0);
            *count += 1;
            if *count > 1 {
                let stem = Path::new(&new_name).file_stem().unwrap().to_string_lossy();
                let extension = Path::new(&new_name).extension().unwrap_or_default().to_string_lossy();
                *new_name = format!("{}_{}.{}", stem, *count - 1, extension);
            }
        }
    }

    // Now rename
    for (path, new_name) in proposed_renames {
        let new_path = Path::new(&path).with_file_name(&new_name);
        if dry_run {
            println!("Would rename {} to {}", path, new_path.display());
        } else {
            fs::rename(&path, &new_path).expect("Failed to rename file");
            println!("Renamed {} to {}", path, new_path.display());
        }
    }
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
    // Take first 100 chars, split on " - " or " by " and take the first part to focus on title
    let mut concise = name.chars().take(100).collect::<String>();
    let separators = [" - ", " by ", " _by ", "_by "];
    for sep in &separators {
        if let Some(pos) = concise.find(sep) {
            concise = concise[..pos].to_string();
            break;
        }
    }
    concise = concise.replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-' && c != '_', "_");
    concise = concise.chars().take(100).collect();
    concise.trim().to_string()
}


