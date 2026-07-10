use crate::config::Config;
use arboard::Clipboard;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::fs::File;
use std::io::Cursor;
use std::path::Path;

pub fn capture_and_save_image(config: &Config, dest_path: &Path) -> Result<String, String> {
    // 1. Initialize clipboard
    let mut clipboard = Clipboard::new().map_err(|e| format!("Failed to open clipboard: {}", e))?;

    // 2. Read image from clipboard
    let image_data = match clipboard.get_image() {
        Ok(img) => img,
        Err(arboard::Error::ContentNotAvailable) => {
            return Err("No image found in clipboard".to_string());
        }
        Err(e) => {
            return Err(format!("Failed to read clipboard image: {}", e));
        }
    };

    // 3. Convert image_data (RGBA) to DynamicImage
    let width = image_data.width as u32;
    let height = image_data.height as u32;

    if width == 0 || height == 0 {
        return Err("Clipboard image has zero dimension".to_string());
    }

    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, image_data.bytes.into_owned())
        .ok_or_else(|| "Failed to convert clipboard data to ImageBuffer".to_string())?;

    let mut img = DynamicImage::ImageRgba8(buffer);

    // 4. Resize if max_dimension is set
    if let Some(max_dim) = config.max_dimension {
        if width > max_dim || height > max_dim {
            img = img.resize(max_dim, max_dim, image::imageops::FilterType::Lanczos3);
        }
    }

    // 5. Output formats and saving
    let output_format = config.output_format.to_lowercase();

    if output_format == "base64" {
        // Compress to JPEG bytes in memory and encode to base64
        let rgb_img = img.to_rgb8();
        let mut jpeg_bytes = Vec::new();
        {
            let mut writer = Cursor::new(&mut jpeg_bytes);
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, config.compress_quality);
            encoder.encode_image(&rgb_img)
                .map_err(|e| format!("Failed to encode image to JPEG: {}", e))?;
        }
        let b64 = base64_encode(&jpeg_bytes);
        return Ok(format!("data:image/jpeg;base64,{}", b64));
    }

    // Default saving to local file
    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create save directory: {}", e))?;
    }

    // Convert to RGB8 for JPEG saving
    let rgb_img = img.to_rgb8();
    let file = File::create(dest_path)
        .map_err(|e| format!("Failed to create image file at {:?}: {}", dest_path, e))?;

    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, config.compress_quality);
    encoder.encode_image(&rgb_img)
        .map_err(|e| format!("Failed to save JPEG image: {}", e))?;

    // Canonicalize path to absolute path
    let abs_path = dest_path.canonicalize()
        .unwrap_or_else(|_| dest_path.to_path_buf());
    let path_str = abs_path.to_string_lossy().to_string();

    match output_format.as_str() {
        "markdown" => Ok(format!("![image]({})", path_str)),
        "html" => Ok(format!("<img src=\"{}\" />", path_str)),
        _ => Ok(path_str), // Default to raw path
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        match chunk.len() {
            3 => {
                let val = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
                result.push(CHARSET[((val >> 18) & 63) as usize] as char);
                result.push(CHARSET[((val >> 12) & 63) as usize] as char);
                result.push(CHARSET[((val >> 6) & 63) as usize] as char);
                result.push(CHARSET[(val & 63) as usize] as char);
            }
            2 => {
                let val = ((chunk[0] as u32) << 8) | (chunk[1] as u32);
                result.push(CHARSET[((val >> 10) & 63) as usize] as char);
                result.push(CHARSET[((val >> 4) & 63) as usize] as char);
                result.push(CHARSET[((val << 2) & 63) as usize] as char);
                result.push('=');
            }
            1 => {
                let val = chunk[0] as u32;
                result.push(CHARSET[((val >> 2) & 63) as usize] as char);
                result.push(CHARSET[((val << 4) & 63) as usize] as char);
                result.push('=');
                result.push('=');
            }
            _ => unreachable!(),
        }
    }
    result
}
