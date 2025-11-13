use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use lopdf::Document;
use std::path::Path;
use std::time::Instant;

use crate::analyzer::analyze_pdf;
use crate::cli::Preset;
use crate::image_optimizer::{optimize_images_in_pdf, create_image_settings_for_preset};
use crate::pdf_reader::{load_pdf, validate_pdf};
use crate::pdf_writer::{save_pdf, create_save_options_for_preset};
use crate::utils::{get_file_size, calculate_compression_ratio, format_bytes};

/// Optimization results
#[derive(Debug)]
pub struct OptimizationResult {
    pub original_size: u64,
    pub optimized_size: u64,
    pub compression_ratio: f64,
    pub images_optimized: usize,
    pub processing_time: std::time::Duration,
}

/// Optimize a single PDF file
pub fn optimize_pdf(
    input_path: &Path,
    output_path: &Path,
    quality: u8,
    preset: &Preset,
    show_progress: bool,
) -> Result<OptimizationResult> {
    let start_time = Instant::now();

    // Set up progress bar
    let pb = if show_progress {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message("Loading PDF...");
        Some(pb)
    } else {
        None
    };

    // Load and validate PDF
    let mut doc = load_pdf(input_path)?;
    validate_pdf(&doc)?;

    if let Some(ref pb) = pb {
        pb.set_message("Analyzing content...");
        pb.inc(10);
    }

    // Analyze the PDF
    let analysis = analyze_pdf(&doc)?;

    if let Some(ref pb) = pb {
        pb.set_message("Optimizing images...");
        pb.inc(20);
    }

    // Create optimization settings
    let image_settings = create_image_settings_for_preset(preset, quality);
    let save_options = create_save_options_for_preset(preset);

    // Optimize images
    let images_optimized = optimize_images_in_pdf(&mut doc, &image_settings)?;

    if let Some(ref pb) = pb {
        pb.set_message("Compressing structure...");
        pb.inc(30);
    }

    // Save optimized PDF
    save_pdf(&mut doc, output_path, &save_options)?;

    if let Some(ref pb) = pb {
        pb.set_message("Finalizing...");
        pb.inc(30);
        pb.finish_with_message("Optimization complete!");
    }

    // Calculate results
    let original_size = get_file_size(input_path)?;
    let optimized_size = get_file_size(output_path)?;
    let compression_ratio = calculate_compression_ratio(original_size, optimized_size);
    let processing_time = start_time.elapsed();

    Ok(OptimizationResult {
        original_size,
        optimized_size,
        compression_ratio,
        images_optimized,
        processing_time,
    })
}

/// Print optimization results
pub fn print_optimization_results(result: &OptimizationResult) {
    println!("\nOptimization Results:");
    println!("===================");
    println!("Original size: {}", format_bytes(result.original_size));
    println!("Optimized size: {}", format_bytes(result.optimized_size));
    println!("Space saved: {:.1}%", result.compression_ratio);
    println!("Images optimized: {}", result.images_optimized);
    println!("Processing time: {:.2}s", result.processing_time.as_secs_f64());

    if result.compression_ratio > 0.0 {
        let saved_bytes = result.original_size - result.optimized_size;
        println!("Bytes saved: {}", format_bytes(saved_bytes));
    }
}