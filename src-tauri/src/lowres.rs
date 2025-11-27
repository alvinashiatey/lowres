use exif::{In, Reader, Tag};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::io::{BufReader, Cursor};
use std::{fs::File, io::BufWriter, path::PathBuf};

type Result<T> = anyhow::Result<T>;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Resample {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

impl From<Resample> for FilterType {
    fn from(r: Resample) -> Self {
        match r {
            Resample::Nearest => FilterType::Nearest,
            Resample::Triangle => FilterType::Triangle,
            Resample::CatmullRom => FilterType::CatmullRom,
            Resample::Gaussian => FilterType::Gaussian,
            Resample::Lanczos3 => FilterType::Lanczos3,
        }
    }
}

impl Display for Resample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Resample::Nearest => "nearest",
            Resample::Triangle => "triangle",
            Resample::CatmullRom => "catmullrom",
            Resample::Gaussian => "gaussian",
            Resample::Lanczos3 => "lanczos3",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ResizeMode {
    /// If one of width/height is missing, preserve aspect. If both provided, use them.
    Auto,
    /// Force exact width×height (may distort); both required.
    Exact,
}

impl Display for ResizeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ResizeMode::Auto => "auto",
            ResizeMode::Exact => "exact",
        };
        write!(f, "{}", s)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LowresConfig {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub mode: Option<ResizeMode>,
    pub filter: Option<Resample>,
    pub block: Option<u32>,
    pub pixel_down_filter: Option<Resample>,
    pub dpi: Option<u32>,
}

pub fn process_image(input: PathBuf, output: PathBuf, config: LowresConfig) -> Result<()> {
    let img = load_image(&input)?;

    let mode = config.mode.unwrap_or(ResizeMode::Auto);
    let filter = config.filter.unwrap_or(Resample::Nearest);
    let pixel_down_filter = config.pixel_down_filter.unwrap_or(Resample::Triangle);
    let dpi = config.dpi.unwrap_or(300);

    let (out_img, _final_w, _final_h) = if let Some(block) = config.block {
        // --- Pixelation path (keeps original WxH) ---
        let down = pixel_down_filter.into();
        let rgba = pixelate(&img, block, down)?;
        let dims = rgba.dimensions();
        (rgba, dims.0, dims.1)
    } else {
        // --- Plain resize path ---
        let (tw, th) = pick_target_size(&img, config.width, config.height, mode)?;
        let filter_type: FilterType = filter.into();
        let resized = resize_image(&img, tw, th, filter_type, mode)?;
        // Convert to RGBA8 for the encoder only once
        let rgba = resized.to_rgba8();
        (rgba, tw, th)
    };

    write_png_with_dpi(&output, out_img, dpi)?;

    Ok(())
}

fn load_image(path: &PathBuf) -> Result<DynamicImage> {
    let data = std::fs::read(path)
        .map_err(|e| anyhow::anyhow!("Failed to read file {:?}: {}", path, e))?;

    // Try to read EXIF orientation
    let orientation = Reader::new()
        .read_from_container(&mut Cursor::new(&data))
        .ok()
        .and_then(|exif| exif.get_field(Tag::Orientation, In::PRIMARY).cloned())
        .and_then(|field| field.value.get_uint(0));

    let img = image::load_from_memory(&data)
        .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

    // Apply orientation
    let img = match orientation {
        Some(2) => img.fliph(),
        Some(3) => img.rotate180(),
        Some(4) => img.flipv(),
        Some(5) => img.rotate90().fliph(),
        Some(6) => img.rotate90(),
        Some(7) => img.rotate270().fliph(),
        Some(8) => img.rotate270(),
        _ => img,
    };

    Ok(img)
}

fn pick_target_size(
    img: &DynamicImage,
    width: Option<u32>,
    height: Option<u32>,
    mode: ResizeMode,
) -> Result<(u32, u32)> {
    let (w0, h0) = img.dimensions();

    match (width, height, mode) {
        (Some(w), Some(h), ResizeMode::Exact) => Ok((w, h)),
        (Some(w), Some(h), ResizeMode::Auto) => Ok((w, h)),

        (Some(w), None, _) => {
            let h = ((h0 as f64) * (w as f64) / (w0 as f64)).round().max(1.0) as u32;
            Ok((w, h))
        }
        (None, Some(h), _) => {
            let w = ((w0 as f64) * (h as f64) / (h0 as f64)).round().max(1.0) as u32;
            Ok((w, h))
        }
        (None, None, _) => Ok((64, 64)),
    }
}

fn resize_image(
    img: &DynamicImage,
    w: u32,
    h: u32,
    filter: FilterType,
    _mode: ResizeMode,
) -> Result<DynamicImage> {
    // Keep as DynamicImage so we can call to_rgba8()
    Ok(img.resize(w, h, filter))
}

/// Pixelate by downscaling to a coarse grid, then upscaling back with Nearest.
/// `block` is the desired block size in source pixels (≈ square size).
/// Optimized version using direct pixel manipulation with parallel processing.
fn pixelate(img: &DynamicImage, block: u32, _down_filter: FilterType) -> Result<RgbaImage> {
    let (w, h) = img.dimensions();
    let b = block.max(1) as usize;

    // Convert to RGBA once at the start
    let rgba = img.to_rgba8();

    // Calculate block grid dimensions
    let blocks_x = (w as usize + b - 1) / b;
    let blocks_y = (h as usize + b - 1) / b;

    // Pre-compute average color for each block in parallel
    let block_colors: Vec<Rgba<u8>> = (0..blocks_y * blocks_x)
        .into_par_iter()
        .map(|idx| {
            let block_y = idx / blocks_x;
            let block_x = idx % blocks_x;

            let x_start = block_x * b;
            let y_start = block_y * b;
            let x_end = ((x_start + b).min(w as usize)) as u32;
            let y_end = ((y_start + b).min(h as usize)) as u32;

            // Average the pixels in this block
            let mut r_sum = 0u32;
            let mut g_sum = 0u32;
            let mut b_sum = 0u32;
            let mut a_sum = 0u32;
            let mut count = 0u32;

            for y in y_start as u32..y_end {
                for x in x_start as u32..x_end {
                    let pixel = rgba.get_pixel(x, y);
                    r_sum += pixel[0] as u32;
                    g_sum += pixel[1] as u32;
                    b_sum += pixel[2] as u32;
                    a_sum += pixel[3] as u32;
                    count += 1;
                }
            }

            if count > 0 {
                Rgba([
                    (r_sum / count) as u8,
                    (g_sum / count) as u8,
                    (b_sum / count) as u8,
                    (a_sum / count) as u8,
                ])
            } else {
                Rgba([0, 0, 0, 255])
            }
        })
        .collect();

    // Create output image by filling each block with its average color
    // Optimized: Use parallel iterator over rows instead of par_bridge on pixels
    let mut buffer = vec![0u8; (w * h * 4) as usize];

    buffer
        .par_chunks_exact_mut((w * 4) as usize)
        .enumerate()
        .for_each(|(y, row)| {
            let block_y = y / b;
            let row_block_start = block_y * blocks_x;

            for x in 0..w as usize {
                let block_x = x / b;
                let color = block_colors[row_block_start + block_x];

                let i = x * 4;
                row[i] = color[0];
                row[i + 1] = color[1];
                row[i + 2] = color[2];
                row[i + 3] = color[3];
            }
        });

    let output = RgbaImage::from_raw(w, h, buffer)
        .ok_or_else(|| anyhow::anyhow!("Failed to create output buffer"))?;

    Ok(output)
}

fn dpi_to_ppm(dpi: u32) -> u32 {
    // PNG pHYs uses pixels-per-meter. 1 inch = 0.0254 m.
    ((dpi as f64) / 0.0254).round() as u32
}

fn write_png_with_dpi(out_path: &PathBuf, rgba: image::RgbaImage, dpi: u32) -> Result<()> {
    use png::{BitDepth, ColorType, Encoder, PixelDimensions, Unit};

    let (w, h) = (rgba.width(), rgba.height());
    let file = File::create(out_path)
        .map_err(|e| anyhow::anyhow!("Failed to create {:?}: {}", out_path, e))?;
    let wtr = BufWriter::new(file);

    let mut encoder = Encoder::new(wtr, w, h);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);

    let ppm = dpi_to_ppm(dpi);
    encoder.set_pixel_dims(Some(PixelDimensions {
        xppu: ppm,
        yppu: ppm,
        unit: Unit::Meter,
    }));

    let mut writer = encoder
        .write_header()
        .map_err(|e| anyhow::anyhow!("PNG header error: {}", e))?;

    writer
        .write_image_data(&rgba)
        .map_err(|e| anyhow::anyhow!("PNG write error: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpi_conversion_is_reasonable() {
        assert_eq!(dpi_to_ppm(300), 11811);
        assert_eq!(dpi_to_ppm(72), 2835);
    }
}
