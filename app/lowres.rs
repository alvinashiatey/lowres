use clap::{Parser, ValueEnum};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage, imageops::FilterType};
use rayon::prelude::*;
use std::fmt::{self, Display};
use std::{fs::File, io::BufWriter, path::PathBuf};

type Result<T> = anyhow::Result<T>;

/// Convert an image to a low-resolution or pixelated PNG and tag DPI.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input image path (jpg, png, etc.)
    #[arg(short, long)]
    input: PathBuf,

    /// Output image path (png recommended, e.g., out.png)
    #[arg(short, long)]
    output: PathBuf,

    /// Target width in pixels (resize mode)
    #[arg(long)]
    width: Option<u32>,

    /// Target height in pixels (resize mode)
    #[arg(long)]
    height: Option<u32>,

    /// Resize behavior (ignored if --block is set)
    #[arg(long, value_enum, default_value_t = ResizeMode::Auto)]
    mode: ResizeMode,

    /// Resampling filter for normal resize (ignored if --block is set)
    #[arg(long, value_enum, default_value_t = Resample::Nearest)]
    filter: Resample,

    /// Pixelation block size in *source pixels*. If set, we pixelate and keep original WxH.
    /// e.g. --block 8 makes ~8×8 squares.
    #[arg(long)]
    block: Option<u32>,

    /// Downscale filter for pixelation (averages colors per block). Upscale is always Nearest.
    #[arg(long, value_enum, default_value_t = Resample::Triangle)]
    pixel_down_filter: Resample,

    /// DPI to set in the output metadata (default 300)
    #[arg(long, default_value_t = 300)]
    dpi: u32,
}

#[derive(Clone, Debug, Copy, ValueEnum, PartialEq, Eq)]
enum Resample {
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

#[derive(Clone, Debug, Copy, ValueEnum, PartialEq, Eq)]
enum ResizeMode {
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

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    let img = load_image(&args.input)?;
    let (orig_w, orig_h) = img.dimensions();

    let (out_img, final_w, final_h) = if let Some(block) = args.block {
        // --- Pixelation path (keeps original WxH) ---
        let down = (args.pixel_down_filter).into();
        let rgba = pixelate(&img, block, down)?;
        let dims = rgba.dimensions();
        (rgba, dims.0, dims.1)
    } else {
        // --- Plain resize path ---
        let (tw, th) = pick_target_size(&img, args.width, args.height, args.mode)?;
        let filter: FilterType = args.filter.into();
        let resized = resize_image(&img, tw, th, filter, args.mode)?;
        // Convert to RGBA8 for the encoder only once
        let rgba = resized.to_rgba8();
        (rgba, tw, th)
    };

    write_png_with_dpi(&args.output, out_img, args.dpi)?;

    println!(
        "Wrote {:?} at {}x{} pixels with {} DPI metadata (mode={}, block={}, filters: resize={}, pixel_down={}). \
Original: {}x{}.",
        args.output,
        final_w,
        final_h,
        args.dpi,
        args.mode,
        args.block
            .map(|b| b.to_string())
            .unwrap_or_else(|| "-".into()),
        args.filter,
        args.pixel_down_filter,
        orig_w,
        orig_h
    );

    Ok(())
}

fn load_image(path: &PathBuf) -> Result<DynamicImage> {
    image::open(path).map_err(|e| anyhow::anyhow!("Failed to open {:?}: {}", path, e))
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
    let mut output: RgbaImage = ImageBuffer::new(w, h);

    output
        .enumerate_pixels_mut()
        .par_bridge()
        .for_each(|(x, y, pixel)| {
            let block_x = (x as usize) / b;
            let block_y = (y as usize) / b;
            let block_idx = block_y * blocks_x + block_x;
            *pixel = block_colors[block_idx];
        });

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
