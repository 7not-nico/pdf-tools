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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        cli::Commands::Optimize { input, output, quality, preset } => {
            // Validate input file
            crate::utils::validate_input_file(&input)?;

            // Perform optimization
            let result = crate::optimizer::optimize_pdf(&input, &output, quality, &preset, true)?;

            // Print results
            crate::optimizer::print_optimization_results(&result);
        }
        cli::Commands::Analyze { input, show_savings } => {
            // Validate input file
            crate::utils::validate_input_file(&input)?;

            // Load and analyze PDF
            let doc = crate::pdf_reader::load_pdf(&input)?;
            crate::pdf_reader::validate_pdf(&doc)?;

            let analysis = crate::analyzer::analyze_pdf(&doc)?;
            crate::analyzer::print_analysis(&analysis, show_savings);

            // Show file size
            let file_size = crate::utils::get_file_size(&input)?;
            println!("File size: {}", crate::utils::format_bytes(file_size));
        }
        cli::Commands::Batch { files, output_dir, threads } => {
            if files.is_empty() {
                eprintln!("Error: No input files specified");
                std::process::exit(1);
            }

            // Validate all input files
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
