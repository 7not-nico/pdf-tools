mod cli;
mod optimizer;
mod pdf_reader;
mod pdf_writer;
mod analyzer;
mod image_optimizer;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use rayon::prelude::*;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(cli::Commands::Optimize { input, output, quality, preset }) => {
            // Resolve input
            let input_path = crate::utils::resolve_input_path(&input.to_str().unwrap())?;
            // Validate input file
            crate::utils::validate_input_file(&input_path)?;

            // Perform optimization
            let result = crate::optimizer::optimize_pdf(&input_path, &output, quality, &preset, true)?;

            // Print results
            crate::optimizer::print_optimization_results(&result);
        }
        Some(cli::Commands::Analyze { input, show_savings }) => {
            // Resolve input
            let input_path = crate::utils::resolve_input_path(&input.to_str().unwrap())?;
            // Validate input file
            crate::utils::validate_input_file(&input_path)?;

            // Load and analyze PDF
            let doc = crate::pdf_reader::load_pdf(&input_path)?;
            crate::pdf_reader::validate_pdf(&doc)?;

            let analysis = crate::analyzer::analyze_pdf(&doc)?;
            crate::analyzer::print_analysis(&analysis, show_savings);

            // Show file size
            let file_size = crate::utils::get_file_size(&input_path)?;
            println!("File size: {}", crate::utils::format_bytes(file_size));
        }
        Some(cli::Commands::Batch { files, output_dir, threads }) => {
            if files.is_empty() {
                eprintln!("Error: No input files specified");
                std::process::exit(1);
            }

            // Resolve and validate all input files
            let resolved_files: Vec<PathBuf> = files.iter().map(|f| crate::utils::resolve_input_path(&f.to_str().unwrap())).collect::<Result<Vec<_>>>()?;
            for (original, resolved) in files.iter().zip(&resolved_files) {
                if let Err(e) = crate::utils::validate_input_file(resolved) {
                    eprintln!("Error with {}: {}", original.display(), e);
                    std::process::exit(1);
                }
            }

            println!("Batch processing {} files with {} threads", resolved_files.len(), threads);

            // Set up rayon thread pool
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .unwrap_or_else(|_| {
                    eprintln!("Warning: Failed to set thread count, using default");
                });

            // Prepare work items
            let work_items: Vec<_> = resolved_files.iter().enumerate().map(|(i, input_file)| {
                let output_file = if let Some(ref dir) = output_dir {
                    dir.join(files[i].file_name().unwrap())
                } else {
                    files[i].with_extension("optimized.pdf")
                };
                (i, input_file.clone(), output_file)
            }).collect();

            // Process files in parallel
            let results: Vec<_> = work_items.into_par_iter().map(|(i, input_file, output_file)| {
                println!("Processing file {}/{}: {}", i + 1, resolved_files.len(), files[i].display());

                match crate::optimizer::optimize_pdf(&input_file, &output_file, 80, &cli::Preset::Web, false) {
                    Ok(result) => {
                        println!("  ✓ Saved {:.1}% ({})",
                                result.compression_ratio,
                                crate::utils::format_bytes(result.original_size - result.optimized_size));
                        Ok(result)
                    }
                    Err(e) => {
                        eprintln!("  ✗ Failed: {}", e);
                        Err(e)
                    }
                }
            }).collect();

            // Calculate totals
            let mut total_original = 0u64;
            let mut total_optimized = 0u64;
            let mut total_images = 0usize;
            let mut successful_files = 0;

            for result in results {
                if let Ok(ref res) = result {
                    total_original += res.original_size;
                    total_optimized += res.optimized_size;
                    total_images += res.images_optimized;
                    successful_files += 1;
                }
            }

            let total_ratio = if total_original > 0 {
                crate::utils::calculate_compression_ratio(total_original, total_optimized)
            } else {
                0.0
            };

            println!("\nBatch Summary:");
            println!("==============");
            println!("Files processed: {}/{}", successful_files, resolved_files.len());
            println!("Total original size: {}", crate::utils::format_bytes(total_original));
            println!("Total optimized size: {}", crate::utils::format_bytes(total_optimized));
            println!("Total space saved: {:.1}%", total_ratio);
            println!("Total images optimized: {}", total_images);
        }
        None => {
            interactive_mode()?;
        }
    }

    Ok(())
}

fn interactive_mode() -> Result<()> {
            for file in &files {
                if let Err(e) = crate::utils::validate_input_file(file) {
                    eprintln!("Error with {}: {}", file.display(), e);
                    std::process::exit(1);
                }
            }

            println!("Batch processing {} files with {} threads", files.len(), threads);

            // Set up rayon thread pool
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .unwrap_or_else(|_| {
                    eprintln!("Warning: Failed to set thread count, using default");
                });

            // Prepare work items
            let work_items: Vec<_> = files.iter().enumerate().map(|(i, input_file)| {
                let output_file = if let Some(ref dir) = output_dir {
                    dir.join(input_file.file_name().unwrap())
                } else {
                    input_file.with_extension("optimized.pdf")
                };
                (i, input_file.clone(), output_file)
            }).collect();

            // Process files in parallel
            let results: Vec<_> = work_items.into_par_iter().map(|(i, input_file, output_file)| {
                println!("Processing file {}/{}: {}", i + 1, files.len(), input_file.display());

                match crate::optimizer::optimize_pdf(&input_file, &output_file, 80, &cli::Preset::Web, false) {
                    Ok(result) => {
                        println!("  ✓ Saved {:.1}% ({})",
                                result.compression_ratio,
                                crate::utils::format_bytes(result.original_size - result.optimized_size));
                        Ok(result)
                    }
                    Err(e) => {
                        eprintln!("  ✗ Failed: {}", e);
                        Err(e)
                    }
                }
            }).collect();

            // Calculate totals
            let mut total_original = 0u64;
            let mut total_optimized = 0u64;
            let mut total_images = 0usize;
            let mut successful_files = 0;

            for result in results {
                if let Ok(ref res) = result {
                    total_original += res.original_size;
                    total_optimized += res.optimized_size;
                    total_images += res.images_optimized;
                    successful_files += 1;
                }
            }

            // Print batch summary
            let total_ratio = if total_original > 0 {
                crate::utils::calculate_compression_ratio(total_original, total_optimized)
            } else {
                0.0
            };

            println!("\nBatch Summary:");
            println!("==============");
            println!("Files processed: {}/{}", successful_files, files.len());
            println!("Total original size: {}", crate::utils::format_bytes(total_original));
            println!("Total optimized size: {}", crate::utils::format_bytes(total_optimized));
            println!("Total space saved: {:.1}%", total_ratio);
            println!("Total images optimized: {}", total_images);
        }
    }

    Ok(())
}

fn interactive_mode() -> Result<()> {
    println!("Interactive mode for pdf-opticompress");
    print!("Choose command (1: Optimize, 2: Analyze, 3: Batch): ");
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    match choice.trim() {
        "1" => {
            print!("Input PDF (URL or local path): ");
            io::stdout().flush().unwrap();
            let mut input_str = String::new();
            io::stdin().read_line(&mut input_str).unwrap();
            let input = crate::utils::resolve_input_path(input_str.trim())?;
            crate::utils::validate_input_file(&input)?;
            print!("Output PDF: ");
            io::stdout().flush().unwrap();
            let mut output_str = String::new();
            io::stdin().read_line(&mut output_str).unwrap();
            let output = PathBuf::from(output_str.trim());
            print!("Quality (0-100, default 80): ");
            io::stdout().flush().unwrap();
            let mut quality_str = String::new();
            io::stdin().read_line(&mut quality_str).unwrap();
            let quality = quality_str.trim().parse().unwrap_or(80);
            print!("Preset (web/print/archive/maximum, default web): ");
            io::stdout().flush().unwrap();
            let mut preset_str = String::new();
            io::stdin().read_line(&mut preset_str).unwrap();
            let preset = match preset_str.trim() {
                "print" => cli::Preset::Print,
                "archive" => cli::Preset::Archive,
                "maximum" => cli::Preset::Maximum,
                _ => cli::Preset::Web,
            };
            let result = crate::optimizer::optimize_pdf(&input, &output, quality, &preset, true)?;
            crate::optimizer::print_optimization_results(&result);
        }
        "2" => {
            print!("Input PDF (URL or local path): ");
            io::stdout().flush().unwrap();
            let mut input_str = String::new();
            io::stdin().read_line(&mut input_str).unwrap();
            let input = crate::utils::resolve_input_path(input_str.trim())?;
            crate::utils::validate_input_file(&input)?;
            let doc = crate::pdf_reader::load_pdf(&input)?;
            crate::pdf_reader::validate_pdf(&doc)?;
            let analysis = crate::analyzer::analyze_pdf(&doc)?;
            crate::analyzer::print_analysis(&analysis, true);
            let file_size = crate::utils::get_file_size(&input)?;
            println!("File size: {}", crate::utils::format_bytes(file_size));
        }
        "3" => {
            print!("Input PDFs (URLs or local paths, space separated): ");
            io::stdout().flush().unwrap();
            let mut files_str = String::new();
            io::stdin().read_line(&mut files_str).unwrap();
            let files: Vec<PathBuf> = files_str.trim().split_whitespace().map(|s| crate::utils::resolve_input_path(s)).collect::<Result<Vec<_>>>()?;
            if files.is_empty() {
                eprintln!("No input files specified");
                return Ok(());
            }
            for file in &files {
                crate::utils::validate_input_file(&file)?;
            }
            print!("Output directory (optional): ");
            io::stdout().flush().unwrap();
            let mut outdir_str = String::new();
            io::stdin().read_line(&mut outdir_str).unwrap();
            let output_dir = if outdir_str.trim().is_empty() { None } else { Some(PathBuf::from(outdir_str.trim())) };
            print!("Threads (default 4): ");
            io::stdout().flush().unwrap();
            let mut threads_str = String::new();
            io::stdin().read_line(&mut threads_str).unwrap();
            let threads = threads_str.trim().parse().unwrap_or(4);
            println!("Batch processing {} files with {} threads", files.len(), threads);
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .unwrap_or_else(|_| eprintln!("Warning: Failed to set thread count, using default"));
            let work_items: Vec<_> = files.iter().enumerate().map(|(i, input_file)| {
                let output_file = if let Some(ref dir) = output_dir {
                    dir.join(format!("optimized_{}.pdf", i))
                } else {
                    PathBuf::from(format!("optimized_{}.pdf", i))
                };
                (i, input_file.clone(), output_file)
            }).collect();
            let results: Vec<_> = work_items.into_par_iter().map(|(i, input_file, output_file)| {
                println!("Processing file {}/{}: {}", i + 1, files.len(), i);
                match crate::optimizer::optimize_pdf(&input_file, &output_file, 80, &cli::Preset::Web, false) {
                    Ok(result) => {
                        println!("  ✓ Saved {:.1}% ({})", result.compression_ratio, crate::utils::format_bytes(result.original_size - result.optimized_size));
                        Ok(result)
                    }
                    Err(e) => {
                        eprintln!("  ✗ Failed: {}", e);
                        Err(e)
                    }
                }
            }).collect();
            let mut total_original = 0u64;
            let mut total_optimized = 0u64;
            let mut total_images = 0usize;
            let mut successful_files = 0;
            for result in results {
                if let Ok(ref res) = result {
                    total_original += res.original_size;
                    total_optimized += res.optimized_size;
                    total_images += res.images_optimized;
                    successful_files += 1;
                }
            }
            let total_ratio = if total_original > 0 { crate::utils::calculate_compression_ratio(total_original, total_optimized) } else { 0.0 };
            println!("\nBatch Summary:\n==============\nFiles processed: {}/{}\nTotal original size: {}\nTotal optimized size: {}\nTotal space saved: {:.1}%\nTotal images optimized: {}", successful_files, files.len(), crate::utils::format_bytes(total_original), crate::utils::format_bytes(total_optimized), total_ratio, total_images);
        }
        "2" => {
            print!("Input PDF: ");
            io::stdout().flush().unwrap();
            let mut input_str = String::new();
            io::stdin().read_line(&mut input_str).unwrap();
            let input = PathBuf::from(input_str.trim());
            crate::utils::validate_input_file(&input)?;
            let doc = crate::pdf_reader::load_pdf(&input)?;
            crate::pdf_reader::validate_pdf(&doc)?;
            let analysis = crate::analyzer::analyze_pdf(&doc)?;
            crate::analyzer::print_analysis(&analysis, true);
            let file_size = crate::utils::get_file_size(&input)?;
            println!("File size: {}", crate::utils::format_bytes(file_size));
        }
        "3" => {
            print!("Input PDFs (space separated): ");
            io::stdout().flush().unwrap();
            let mut files_str = String::new();
            io::stdin().read_line(&mut files_str).unwrap();
            let files: Vec<PathBuf> = files_str.trim().split_whitespace().map(PathBuf::from).collect();
            if files.is_empty() {
                eprintln!("No input files specified");
                return Ok(());
            }
            for file in &files {
                crate::utils::validate_input_file(file)?;
            }
            print!("Output directory (optional): ");
            io::stdout().flush().unwrap();
            let mut outdir_str = String::new();
            io::stdin().read_line(&mut outdir_str).unwrap();
            let output_dir = if outdir_str.trim().is_empty() { None } else { Some(PathBuf::from(outdir_str.trim())) };
            print!("Threads (default 4): ");
            io::stdout().flush().unwrap();
            let mut threads_str = String::new();
            io::stdin().read_line(&mut threads_str).unwrap();
            let threads = threads_str.trim().parse().unwrap_or(4);
            println!("Batch processing {} files with {} threads", files.len(), threads);
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .unwrap_or_else(|_| eprintln!("Warning: Failed to set thread count, using default"));
            let work_items: Vec<_> = files.iter().enumerate().map(|(i, input_file)| {
                let output_file = if let Some(ref dir) = output_dir {
                    dir.join(input_file.file_name().unwrap())
                } else {
                    input_file.with_extension("optimized.pdf")
                };
                (i, input_file.clone(), output_file)
            }).collect();
            let results: Vec<_> = work_items.into_par_iter().map(|(i, input_file, output_file)| {
                println!("Processing file {}/{}: {}", i + 1, files.len(), input_file.display());
                match crate::optimizer::optimize_pdf(&input_file, &output_file, 80, &cli::Preset::Web, false) {
                    Ok(result) => {
                        println!("  ✓ Saved {:.1}% ({})", result.compression_ratio, crate::utils::format_bytes(result.original_size - result.optimized_size));
                        Ok(result)
                    }
                    Err(e) => {
                        eprintln!("  ✗ Failed: {}", e);
                        Err(e)
                    }
                }
            }).collect();
            let mut total_original = 0u64;
            let mut total_optimized = 0u64;
            let mut total_images = 0usize;
            let mut successful_files = 0;
            for result in results {
                if let Ok(ref res) = result {
                    total_original += res.original_size;
                    total_optimized += res.optimized_size;
                    total_images += res.images_optimized;
                    successful_files += 1;
                }
            }
            let total_ratio = if total_original > 0 { crate::utils::calculate_compression_ratio(total_original, total_optimized) } else { 0.0 };
            println!("\nBatch Summary:\n==============\nFiles processed: {}/{}\nTotal original size: {}\nTotal optimized size: {}\nTotal space saved: {:.1}%\nTotal images optimized: {}", successful_files, files.len(), crate::utils::format_bytes(total_original), crate::utils::format_bytes(total_optimized), total_ratio, total_images);
        }
        _ => println!("Invalid choice"),
    }
    Ok(())
}
