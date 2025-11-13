use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageFormat};
use lopdf::{Document, Object, Stream};

/// Image optimization settings
#[derive(Clone)]
pub struct ImageSettings {
    pub jpeg_quality: u8, // 0-100
    pub enable_png_optimization: bool,
    pub max_dimension: Option<u32>, // Maximum width/height, None = no limit
}

impl Default for ImageSettings {
    fn default() -> Self {
        Self {
            jpeg_quality: 80,
            enable_png_optimization: true,
            max_dimension: None,
        }
    }
}

/// Create image settings based on preset
pub fn create_image_settings_for_preset(preset: &crate::cli::Preset, quality: u8) -> ImageSettings {
    match preset {
        crate::cli::Preset::Web => ImageSettings {
            jpeg_quality: quality,
            enable_png_optimization: true,
            max_dimension: Some(1920), // Limit for web viewing
        },
        crate::cli::Preset::Print => ImageSettings {
            jpeg_quality: quality.max(85), // Higher quality for print
            enable_png_optimization: true,
            max_dimension: None, // No limit for print
        },
        crate::cli::Preset::Archive => ImageSettings {
            jpeg_quality: quality,
            enable_png_optimization: true,
            max_dimension: None,
        },
        crate::cli::Preset::Maximum => ImageSettings {
            jpeg_quality: quality.min(70), // More aggressive compression
            enable_png_optimization: true,
            max_dimension: Some(1024), // Smaller for maximum compression
        },
    }
}

/// Optimize images in a PDF document
pub fn optimize_images_in_pdf(doc: &mut Document, settings: &ImageSettings) -> Result<usize> {
    let mut optimized_count = 0;

    // Get all objects that might contain images
    let objects = doc.objects.clone();

    for (id, obj) in objects {
        if let Object::Stream(ref stream) = obj {
            // Check if this is an image
            if is_image_stream(stream) {
                if let Some(optimized_stream) = optimize_image_stream(stream, settings)? {
                    doc.objects.insert(id, Object::Stream(optimized_stream));
                    optimized_count += 1;
                }
            }
        }
    }

    Ok(optimized_count)
}

/// Check if a stream contains an image
fn is_image_stream(stream: &Stream) -> bool {
    if let Ok(subtype) = stream.dict.get(b"Subtype") {
        if let lopdf::Object::Name(ref name) = subtype {
            return name == b"Image";
        }
    }
    false
}

/// Optimize an image stream
fn optimize_image_stream(stream: &Stream, settings: &ImageSettings) -> Result<Option<Stream>> {
    // Extract image data
    let image_data = &stream.content;

    // Determine image format
    let format = detect_image_format(stream)?;

    match format {
        ImageFormat::Jpeg => {
            let optimized = optimize_jpeg_image(image_data, settings)?;
            Ok(Some(create_optimized_stream(stream, &optimized)))
        }
        ImageFormat::Png => {
            if settings.enable_png_optimization {
                let optimized = optimize_png_image(image_data, settings)?;
                Ok(Some(create_optimized_stream(stream, &optimized)))
            } else {
                Ok(None) // No optimization needed
            }
        }
        _ => {
            // For other formats, try to convert to JPEG
            let optimized = convert_and_optimize_image(image_data, format, settings)?;
            Ok(Some(create_optimized_stream(stream, &optimized)))
        }
    }
}

/// Detect image format from stream dictionary
fn detect_image_format(stream: &Stream) -> Result<ImageFormat> {
    // Check filter
    if let Ok(filter) = stream.dict.get(b"Filter") {
        if let lopdf::Object::Name(ref name) = filter {
            match name.as_slice() {
                b"DCTDecode" => return Ok(ImageFormat::Jpeg),
                b"FlateDecode" => {
                    // Could be PNG or other, check for PNG signature
                    if stream.content.starts_with(b"\x89PNG") {
                        return Ok(ImageFormat::Png);
                    }
                }
                _ => {}
            }
        }
    }

    // Check for PNG signature in content
    if stream.content.starts_with(b"\x89PNG") {
        return Ok(ImageFormat::Png);
    }

    // Default to JPEG for DCTDecode or unknown
    Ok(ImageFormat::Jpeg)
}

/// Optimize JPEG image
fn optimize_jpeg_image(data: &[u8], settings: &ImageSettings) -> Result<Vec<u8>> {
    let img = image::load_from_memory_with_format(data, ImageFormat::Jpeg)
        .context("Failed to load JPEG image")?;

    // Resize if needed
    let img = resize_image_if_needed(img, settings);

    // Re-encode with specified quality
    let mut output = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut output), ImageFormat::Jpeg)
        .context("Failed to encode JPEG")?;

    Ok(output)
}

/// Optimize PNG image using oxipng
fn optimize_png_image(data: &[u8], _settings: &ImageSettings) -> Result<Vec<u8>> {
    use oxipng::{optimize_from_memory, Options};

    let options = Options::default();
    optimize_from_memory(data, &options)
        .context("Failed to optimize PNG with oxipng")
}

/// Convert and optimize other image formats
fn convert_and_optimize_image(data: &[u8], format: ImageFormat, settings: &ImageSettings) -> Result<Vec<u8>> {
    let img = image::load_from_memory_with_format(data, format)
        .context("Failed to load image")?;

    // Resize if needed
    let img = resize_image_if_needed(img, settings);

    // Convert to JPEG
    let mut output = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut output), ImageFormat::Jpeg)
        .context("Failed to encode image as JPEG")?;

    Ok(output)
}

/// Resize image if it exceeds maximum dimensions
fn resize_image_if_needed(img: DynamicImage, settings: &ImageSettings) -> DynamicImage {
    if let Some(max_dim) = settings.max_dimension {
        let (width, height) = img.dimensions();
        if width > max_dim || height > max_dim {
            let aspect_ratio = width as f32 / height as f32;
            let (new_width, new_height) = if width > height {
                (max_dim, (max_dim as f32 / aspect_ratio) as u32)
            } else {
                ((max_dim as f32 * aspect_ratio) as u32, max_dim)
            };

            return img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
        }
    }
    img
}

/// Create an optimized stream with new content
fn create_optimized_stream(original: &Stream, new_content: &[u8]) -> Stream {
    let mut new_stream = original.clone();
    new_stream.content = new_content.to_vec();

    // Update length in dictionary
    new_stream.dict.set("Length", new_content.len() as i64);

    new_stream
}