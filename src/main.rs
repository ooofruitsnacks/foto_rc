use clap::{Parser, ValueEnum};
use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType as PngFilterType, PngEncoder};
use image::{DynamicImage, ExtendedColorType, ImageEncoder, RgbImage};
use imagepipe::simple_decode_8bit;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum OutFormat {
    Jpeg,
    Png,
    Avif,
    Keep,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long, value_enum, default_value_t = OutFormat::Keep)]
    format: OutFormat,

    #[arg(short, long, default_value_t = 85)]
    quality: u8,

    #[arg(short, long, default_value_t = 100)]
    scale: u32,

    #[arg(short, long)]
    target_size: Option<String>,

    #[arg(short, long)]
    recursive: bool,

    #[arg(long)]
    overwrite: bool,
}

const SOURCE_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp", "ico",
];

const RAW_EXTS: &[&str] = &["raw", "nef", "dng"];

fn main() {
    let args = Args::parse();

    if !(10..=100).contains(&args.quality) {
        eprintln!("Error: --quality must be between 10 and 100%.");
        std::process::exit(1);
    }
    if !(10..=100).contains(&args.scale) {
        eprintln!("Error: --scale must be between 10 and 100%.");
        std::process::exit(1);
    }

    let target_bytes: Option<u64> = match &args.target_size {
        Some(s) => match parse_size(s) {
            Ok(b) => Some(b),
            Err(e) => {
                eprintln!("Error parsing --target-size: {}", e);
                std::process::exit(1);
            }
        },
        None => None,
    };

    if !args.input.is_dir() {
        eprintln!("Error: '{}' is not a valid directory.", args.input.display());
        std::process::exit(1);
    }

    let output_dir = args
        .output
        .clone()
        .unwrap_or_else(|| args.input.join("resized"));
    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("Error creating output folder '{}': {}", output_dir.display(), e);
        std::process::exit(1);
    }

    let files = collect_images(&args.input, args.recursive);
    if files.is_empty() {
        eprintln!("No supported images found in '{}'.", args.input.display());
        std::process::exit(1);
    }

    println!("Found {} image(s). Processing...\n", files.len());

    let mut success = 0usize;
    let mut failed = 0usize;

    for path in &files {
        match process_image(path, &output_dir, &args, target_bytes) {
            Ok((out_path, orig_size, new_size)) => {
                success += 1;
                let saved_pct = if orig_size > 0 {
                    100.0 - (new_size as f64 / orig_size as f64 * 100.0)
                } else {
                    0.0
                };
                println!(
                    "  {} ({} KB) -> {} ({} KB, -{:.1}%)",
                    path.file_name().unwrap().to_string_lossy(),
                    orig_size / 1024,
                    out_path.file_name().unwrap().to_string_lossy(),
                    new_size / 1024,
                    saved_pct
                );
            }
            Err(e) => {
                failed += 1;
                eprintln!("  FAILED: {} — {}", path.display(), e);
            }
        }
    }

    println!("\nDone. {} succeeded, {} failed.", success, failed);
    println!("Output folder: {}", output_dir.display());
}

fn process_image(
    path: &Path,
    output_dir: &Path,
    args: &Args,
    target_bytes: Option<u64>,
) -> Result<(PathBuf, u64, u64), String> {
    let orig_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let img = if is_raw_file(path) {
        decode_raw(path)?
    } else {
        image::open(path).map_err(|e| format!("could not open image: {}", e))?
    };

    let out_format = resolve_format(args.format, path);
    let ext = match out_format {
        OutFormat::Jpeg => "jpg",
        OutFormat::Png => "png",
        OutFormat::Avif => "avif",
        OutFormat::Keep => unreachable!(),
    };

    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let mut out_path = output_dir.join(format!("{}.{}", stem, ext));
    if !args.overwrite {
        out_path = unique_path(out_path);
    }

    let bytes = if let Some(target) = target_bytes {
        let (bytes, used_scale, used_quality) = fit_to_target_size(&img, out_format, target)?;
        if bytes.len() as u64 > target {
            eprintln!(
                "    Note: '{}' could not be reduced below target even at scale {}% / quality {} — saved smallest possible version instead.",
                path.file_name().unwrap().to_string_lossy(),
                used_scale,
                used_quality
            );
        }
        bytes
    } else {
        let resized = if args.scale != 100 {
            resize_by_percent(&img, args.scale)
        } else {
            img
        };
        encode_image(&resized, out_format, args.quality)?
    };

    fs::write(&out_path, &bytes).map_err(|e| format!("failed to write output: {}", e))?;

    Ok((out_path.clone(), orig_size, bytes.len() as u64))
}

fn is_raw_file(path: &Path) -> bool {
    path.extension()
        .map(|e| RAW_EXTS.contains(&e.to_string_lossy().to_lowercase().as_str()))
        .unwrap_or(false)
}

fn decode_raw(path: &Path) -> Result<DynamicImage, String> {
    let decoded =
        simple_decode_8bit(path, 0, 0).map_err(|e| format!("could not decode raw file: {}", e))?;

    let buffer = RgbImage::from_raw(decoded.width as u32, decoded.height as u32, decoded.data)
        .ok_or_else(|| "raw decoder returned an invalid pixel buffer".to_string())?;

    Ok(DynamicImage::ImageRgb8(buffer))
}

fn resolve_format(requested: OutFormat, source: &Path) -> OutFormat {
    match requested {
        OutFormat::Keep => {
            let ext = source
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            match ext.as_str() {
                "jpg" | "jpeg" => OutFormat::Jpeg,
                "png" => OutFormat::Png,
                "raw" | "nef" | "dng" => OutFormat::Jpeg,
                _ => OutFormat::Png,
            }
        }
        other => other,
    }
}

fn resize_by_percent(img: &DynamicImage, percent: u32) -> DynamicImage {
    let (w, h) = (img.width(), img.height());
    let new_w = ((w as f64) * (percent as f64) / 100.0).round().max(1.0) as u32;
    let new_h = ((h as f64) * (percent as f64) / 100.0).round().max(1.0) as u32;
    img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3)
}

fn encode_image(img: &DynamicImage, format: OutFormat, quality: u8) -> Result<Vec<u8>, String> {
    let mut buf: Vec<u8> = Vec::new();
    match format {
        OutFormat::Jpeg => {
            let rgb = img.to_rgb8();
            JpegEncoder::new_with_quality(&mut buf, quality)
                .write_image(rgb.as_raw(), rgb.width(), rgb.height(), ExtendedColorType::Rgb8)
                .map_err(|e| e.to_string())?;
        }
        OutFormat::Png => {
            let rgba = img.to_rgba8();
            PngEncoder::new_with_quality(&mut buf, CompressionType::Best, PngFilterType::Adaptive)
                .write_image(rgba.as_raw(), rgba.width(), rgba.height(), ExtendedColorType::Rgba8)
                .map_err(|e| e.to_string())?;
        }
        OutFormat::Avif => {
            let rgba = img.to_rgba8();
            AvifEncoder::new_with_speed_quality(&mut buf, 6, quality)
                .write_image(rgba.as_raw(), rgba.width(), rgba.height(), ExtendedColorType::Rgba8)
                .map_err(|e| e.to_string())?;
        }
        OutFormat::Keep => unreachable!(),
    }
    Ok(buf)
}

fn fit_to_target_size(
    img: &DynamicImage,
    format: OutFormat,
    target: u64,
) -> Result<(Vec<u8>, u32, u8), String> {
    let scales: [u32; 10] = [100, 90, 80, 70, 60, 50, 40, 30, 20, 10];
    let qualities: [u8; 18] = [
        95, 90, 85, 80, 75, 70, 65, 60, 55, 50, 45, 40, 35, 30, 25, 20, 15, 10,
    ];

    let mut smallest: Option<(Vec<u8>, u32, u8)> = None;

    for &scale in &scales {
        let resized = if scale == 100 {
            img.clone()
        } else {
            resize_by_percent(img, scale)
        };

        if format == OutFormat::Png {
            let bytes = encode_image(&resized, format, 100)?;
            let len = bytes.len() as u64;
            if smallest.is_none() || len < smallest.as_ref().unwrap().0.len() as u64 {
                smallest = Some((bytes.clone(), scale, 100));
            }
            if len <= target {
                return Ok((bytes, scale, 100));
            }
            continue;
        }

        for &q in &qualities {
            let bytes = encode_image(&resized, format, q)?;
            let len = bytes.len() as u64;
            if smallest.is_none() || len < smallest.as_ref().unwrap().0.len() as u64 {
                smallest = Some((bytes.clone(), scale, q));
            }
            if len <= target {
                return Ok((bytes, scale, q));
            }
        }
    }

    smallest.ok_or_else(|| "failed to encode image".to_string())
}

fn parse_size(input: &str) -> Result<u64, String> {
    let cleaned = input.trim().to_lowercase();
    let cleaned = cleaned
        .trim_start_matches("under")
        .trim_start_matches('<')
        .trim();

    let split_at = cleaned
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(cleaned.len());
    let (num_part, unit_part) = cleaned.split_at(split_at);

    let num: f64 = num_part
        .trim()
        .parse()
        .map_err(|_| format!("could not parse a number from '{}'", input))?;

    let unit = unit_part.trim();
    let bytes = match unit {
        "" | "b" | "byte" | "bytes" => num,
        "kb" | "k" | "kilobyte" | "kilobytes" => num * 1024.0,
        "mb" | "m" | "megabyte" | "megabytes" => num * 1024.0 * 1024.0,
        other => return Err(format!("unrecognized size unit '{}'", other)),
    };

    Ok(bytes.round() as u64)
}

fn collect_images(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_images_inner(dir, recursive, &mut out);
    out
}

fn collect_images_inner(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_images_inner(&path, recursive, out);
            }
            continue;
        }
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if SOURCE_EXTS.contains(&ext.as_str()) || RAW_EXTS.contains(&ext.as_str()) {
                out.push(path);
            }
        }
    }
}

fn unique_path(mut path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let stem = path.file_stem().unwrap().to_string_lossy().to_string();
    let ext = path.extension().unwrap().to_string_lossy().to_string();
    let parent = path.parent().unwrap().to_path_buf();
    let mut n = 1;
    loop {
        let candidate = parent.join(format!("{}_{}.{}", stem, n, ext));
        if !candidate.exists() {
            path = candidate;
            break;
        }
        n += 1;
    }
    path
}
