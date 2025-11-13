use anyhow::Result;
use lopdf::Document;

/// Analysis results for a PDF document
#[derive(Debug)]
pub struct PdfAnalysis {
    pub total_objects: usize,
    pub image_count: usize,
    pub font_count: usize,
    pub text_objects: usize,
    pub estimated_savings: EstimatedSavings,
    pub content_breakdown: ContentBreakdown,
}

#[derive(Debug)]
pub struct EstimatedSavings {
    pub image_compression: f64, // Percentage
    pub structure_optimization: f64, // Percentage
    pub total_estimated: f64, // Percentage
}

#[derive(Debug)]
pub struct ContentBreakdown {
    pub images_size: u64,
    pub fonts_size: u64,
    pub text_size: u64,
    pub other_size: u64,
    pub total_size: u64,
}

/// Analyze a PDF document and calculate optimization potential
pub fn analyze_pdf(doc: &Document) -> Result<PdfAnalysis> {
    let mut image_count = 0;
    let mut font_count = 0;
    let mut text_objects = 0;
    let mut images_size = 0u64;
    let mut fonts_size = 0u64;
    let mut text_size = 0u64;
    let mut other_size = 0u64;

    // Iterate through all objects to analyze content
    for (_, obj) in &doc.objects {
        match obj {
            lopdf::Object::Stream(ref stream) => {
                // Check if this is an image
                if let Ok(subtype) = stream.dict.get(b"Subtype") {
                    if let lopdf::Object::Name(ref name) = subtype {
                        if name == b"Image" {
                            image_count += 1;
                            images_size += stream.content.len() as u64;
                        }
                    }
                }

                // Check if this is a font stream
                if let Ok(obj_type) = stream.dict.get(b"Type") {
                    if let lopdf::Object::Name(ref name) = obj_type {
                        if name == b"Font" {
                            font_count += 1;
                            fonts_size += stream.content.len() as u64;
                        }
                    }
                }

                // Estimate text content (rough heuristic)
                if stream.dict.get(b"Length").is_ok() {
                    let content = &stream.content;
                    if content.windows(4).any(|w| w == b"BT\n") {
                        text_objects += 1;
                        text_size += content.len() as u64;
                    }
                }
            }
            lopdf::Object::Dictionary(ref dict) => {
                // Check for font dictionaries
                if let Ok(obj_type) = dict.get(b"Type") {
                    if let lopdf::Object::Name(ref name) = obj_type {
                        if name == b"Font" {
                            font_count += 1;
                        }
                    }
                }
            }
            _ => {
                // Other objects
                other_size += 100; // Rough estimate
            }
        }
    }

    let total_objects = doc.objects.len();
    let total_size = images_size + fonts_size + text_size + other_size;

    // Estimate savings potential
    let image_compression = if image_count > 0 {
        // Assume 30-70% savings on images depending on current compression
        50.0
    } else {
        0.0
    };

    let structure_optimization = if total_objects > 100 {
        // Object streams can save 11-38% on large documents
        25.0
    } else {
        10.0
    };

    let total_estimated = (image_compression * 0.6) + (structure_optimization * 0.4);

    Ok(PdfAnalysis {
        total_objects,
        image_count,
        font_count,
        text_objects,
        estimated_savings: EstimatedSavings {
            image_compression,
            structure_optimization,
            total_estimated,
        },
        content_breakdown: ContentBreakdown {
            images_size,
            fonts_size,
            text_size,
            other_size,
            total_size,
        },
    })
}

/// Print analysis results in a human-readable format
pub fn print_analysis(analysis: &PdfAnalysis, show_savings: bool) {
    println!("PDF Analysis Results:");
    println!("====================");
    println!("Total objects: {}", analysis.total_objects);
    println!("Images: {}", analysis.image_count);
    println!("Fonts: {}", analysis.font_count);
    println!("Text objects: {}", analysis.text_objects);
    println!();

    println!("Content Breakdown:");
    println!("Images: {}", crate::utils::format_bytes(analysis.content_breakdown.images_size));
    println!("Fonts: {}", crate::utils::format_bytes(analysis.content_breakdown.fonts_size));
    println!("Text: {}", crate::utils::format_bytes(analysis.content_breakdown.text_size));
    println!("Other: {}", crate::utils::format_bytes(analysis.content_breakdown.other_size));
    println!("Total: {}", crate::utils::format_bytes(analysis.content_breakdown.total_size));
    println!();

    if show_savings {
        println!("Estimated Savings:");
        println!("Image compression: {:.1}%", analysis.estimated_savings.image_compression);
        println!("Structure optimization: {:.1}%", analysis.estimated_savings.structure_optimization);
        println!("Total estimated: {:.1}%", analysis.estimated_savings.total_estimated);
    }
}