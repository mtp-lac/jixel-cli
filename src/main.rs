//! jixel-cli - Command-line JPEG XL encoder using the jixel library

#![forbid(unsafe_code)]

mod cli;

use std::num::NonZero;
use std::thread::available_parallelism;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::Parser;
use image::DynamicImage;
use jixel::{
    ColorSpace, EncodeConfig, Orientation, encode_fast_lossless, encode_fast_lossless_u16,
    encode_image, encode_image_with_alpha, encode_jpeg_lossless,
};

use cli::Args;

fn main() -> Result<()> {
    let args = Args::parse();
    run(&args)
}

fn run(args: &Args) -> Result<()> {
    let start = Instant::now();

    let output_path = args.output.clone().unwrap_or_else(|| {
        let mut p = args.input.clone();
        p.set_extension("jxl");
        p
    });

    if !args.quiet {
        eprintln!(
            "jixel-cli: {} -> {}",
            args.input.display(),
            output_path.display()
        );
    }

    // JPEG lossless transcode reads raw file bytes, no image decode needed.
    if args.jpeg_lossless {
        let jpeg_bytes = std::fs::read(&args.input)
            .with_context(|| format!("Failed to read JPEG: {}", args.input.display()))?;
        if !jpeg_bytes.starts_with(&[0xFF, 0xD8]) {
            bail!("--jpeg-lossless requires a JPEG input (missing SOI marker)");
        }
        let jxl_bytes = encode_jpeg_lossless(&jpeg_bytes).context("JPEG transcode failed")?;
        std::fs::write(&output_path, &jxl_bytes)
            .with_context(|| format!("Failed to write output: {}", output_path.display()))?;
        print_stats(args, start.elapsed(), jxl_bytes.len());
        return Ok(());
    }

    let img = image::open(&args.input)
        .with_context(|| format!("Failed to open image: {}", args.input.display()))?;

    let icc = match &args.icc_profile {
        Some(path) => Some(
            std::fs::read(path)
                .with_context(|| format!("Failed to read ICC profile: {}", path.display()))?,
        ),
        None => None,
    };

    let orientation = match args.orientation {
        Some(v) => Some(Orientation::from_exif(v).context("Invalid orientation value")?),
        None => None,
    };

    let jxl_bytes = if args.fast_lossless {
        encode_fast_lossless_dynamic(&img, args, icc, orientation)
            .with_context(|| format!("Fast-lossless encode failed: {}", args.input.display()))?
    } else {
        let mut config = EncodeConfig::default()
            .with_lossless(args.lossless)
            .with_speed(args.speed.into())
            .with_decoding_speed(args.decoding_speed.into())
            .with_progressive(args.progressive)
            .with_patches(!args.no_patches)
            .with_splines(args.splines)
            .with_color_encoding(args.color_space.to_color_encoding());

        if !args.lossless {
            config = config.with_quality(args.quality as f32);
        }
        if let Some(lm) = args.lossy_modular {
            config = config.with_lossy_modular(lm.into());
        }
        if let Some(profile) = icc {
            config = config.with_icc_profile(profile);
        }
        if let Some(orient) = orientation {
            config = config.with_orientation(orient);
        }
        let threads = if args.threads == 0 {
            available_parallelism()
                .unwrap_or_else(|_| NonZero::new(1).unwrap())
                .get()
        } else {
            args.threads
        };
        config = config.with_num_threads(threads);

        if !args.quiet {
            let mode = if args.lossless { "Lossless" } else { "Lossy" };
            eprintln!(
                "  Mode: {mode} (quality={}, speed={:?}, threads={threads})",
                if args.lossless { 100 } else { args.quality },
                args.speed,
            );
        }

        encode_image_dynamic(&img, &config, args.no_alpha)
            .with_context(|| format!("Encode failed: {}", args.input.display()))?
    };

    std::fs::write(&output_path, &jxl_bytes)
        .with_context(|| format!("Failed to write output: {}", output_path.display()))?;

    if args.verbose {
        eprintln!(
            "  Input: {}x{} {:?}",
            img.width(),
            img.height(),
            img.color()
        );
    }
    print_stats(args, start.elapsed(), jxl_bytes.len());
    Ok(())
}

/// Standard encoder path (VarDCT lossy / modular lossless) for all pixel formats.
fn encode_image_dynamic(
    img: &DynamicImage,
    config: &EncodeConfig,
    no_alpha: bool,
) -> Result<Vec<u8>> {
    let width = img.width() as usize;
    let height = img.height() as usize;

    let bytes = match img {
        DynamicImage::ImageLuma8(buf) => {
            jixel::encode_image_gray(buf.as_raw(), width, height, config)?
        }
        DynamicImage::ImageLumaA8(buf) => {
            jixel::encode_image_gray_alpha(buf.as_raw(), width, height, config)?
        }
        DynamicImage::ImageRgb8(buf) => encode_image(buf.as_raw(), width, height, config)?,
        DynamicImage::ImageRgba8(buf) => {
            if no_alpha {
                let rgb = strip_alpha(buf.as_raw());
                encode_image(&rgb, width, height, config)?
            } else {
                encode_image_with_alpha(buf.as_raw(), width, height, config)?
            }
        }
        DynamicImage::ImageLuma16(buf) => {
            jixel::encode_image_gray_16bit(buf.as_raw(), width, height, config)?
        }
        DynamicImage::ImageLumaA16(buf) => {
            jixel::encode_image_gray_alpha_16bit(buf.as_raw(), width, height, config)?
        }
        DynamicImage::ImageRgb16(buf) => {
            jixel::encode_image_16bit(buf.as_raw(), width, height, config)?
        }
        DynamicImage::ImageRgba16(buf) => {
            if no_alpha {
                let rgb = strip_alpha(buf.as_raw());
                jixel::encode_image_16bit(&rgb, width, height, config)?
            } else {
                jixel::encode_image_with_alpha_16bit(buf.as_raw(), width, height, config)?
            }
        }
        DynamicImage::ImageRgb32F(buf) => {
            reject_lossless(config)?;
            jixel::encode_image_f32(buf.as_raw(), width, height, config)?
        }
        DynamicImage::ImageRgba32F(buf) => {
            reject_lossless(config)?;
            if no_alpha {
                let rgb = strip_alpha(buf.as_raw());
                jixel::encode_image_f32(&rgb, width, height, config)?
            } else {
                jixel::encode_image_with_alpha_f32(buf.as_raw(), width, height, config)?
            }
        }
        // DynamicImage is #[non_exhaustive]; refuse to silently
        // re-quantize any pixel format the encoder cannot handle natively.
        other => bail!("unsupported pixel format: {:?}", other.color()),
    };
    Ok(bytes)
}

/// Fast-lossless path. Only integer formats are supported by this encoder.
fn encode_fast_lossless_dynamic(
    img: &DynamicImage,
    args: &Args,
    icc: Option<Vec<u8>>,
    orientation: Option<Orientation>,
) -> Result<Vec<u8>> {
    let width = img.width() as usize;
    let height = img.height() as usize;

    let mut meta = args.color_space.to_fl_meta();
    if let Some(profile) = icc {
        meta.icc = Some(profile);
    }
    if let Some(orient) = orientation {
        meta.orientation = orient;
    }

    let bytes = match img {
        DynamicImage::ImageLuma8(buf) => {
            encode_fast_lossless(buf.as_raw(), width, height, ColorSpace::Gray, false, &meta)?
        }
        DynamicImage::ImageLumaA8(buf) => {
            encode_fast_lossless(buf.as_raw(), width, height, ColorSpace::Gray, true, &meta)?
        }
        DynamicImage::ImageRgb8(buf) => {
            encode_fast_lossless(buf.as_raw(), width, height, ColorSpace::Rgb, false, &meta)?
        }
        DynamicImage::ImageRgba8(buf) => {
            if args.no_alpha {
                let rgb = strip_alpha(buf.as_raw());
                encode_fast_lossless(&rgb, width, height, ColorSpace::Rgb, false, &meta)?
            } else {
                encode_fast_lossless(buf.as_raw(), width, height, ColorSpace::Rgb, true, &meta)?
            }
        }
        DynamicImage::ImageLuma16(buf) => encode_fast_lossless_u16(
            buf.as_raw(),
            width,
            height,
            ColorSpace::Gray,
            false,
            16,
            &meta,
        )?,
        DynamicImage::ImageLumaA16(buf) => encode_fast_lossless_u16(
            buf.as_raw(),
            width,
            height,
            ColorSpace::Gray,
            true,
            16,
            &meta,
        )?,
        DynamicImage::ImageRgb16(buf) => encode_fast_lossless_u16(
            buf.as_raw(),
            width,
            height,
            ColorSpace::Rgb,
            false,
            16,
            &meta,
        )?,
        DynamicImage::ImageRgba16(buf) => {
            if args.no_alpha {
                let rgb = strip_alpha(buf.as_raw());
                encode_fast_lossless_u16(&rgb, width, height, ColorSpace::Rgb, false, 16, &meta)?
            } else {
                encode_fast_lossless_u16(
                    buf.as_raw(),
                    width,
                    height,
                    ColorSpace::Rgb,
                    true,
                    16,
                    &meta,
                )?
            }
        }
        DynamicImage::ImageRgb32F(_) | DynamicImage::ImageRgba32F(_) => {
            bail!(
                "fast-lossless does not support float images; use the regular --lossless encoder"
            );
        }
        // DynamicImage is #[non_exhaustive]; fast-lossless must never
        // silently re-quantize pixels, so reject anything unlisted.
        other => bail!(
            "fast-lossless: unsupported pixel format: {:?}",
            other.color()
        ),
    };
    Ok(bytes)
}

/// The float entry points are lossy-only.
fn reject_lossless(config: &EncodeConfig) -> Result<()> {
    if config.lossless {
        bail!("lossless is not supported for float images; convert to an integer pixel format");
    }
    Ok(())
}

/// Drop the 4th channel from interleaved RGBA samples (u8/u16/f32).
fn strip_alpha<T: Copy>(input: &[T]) -> Vec<T> {
    input
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|c| [c[0], c[1], c[2]])
        .collect()
}

fn print_stats(args: &Args, elapsed: std::time::Duration, output_size: usize) {
    if args.quiet {
        return;
    }
    let input_size = std::fs::metadata(&args.input).map(|m| m.len()).unwrap_or(0);
    let ratio = if input_size > 0 {
        output_size as f64 / input_size as f64
    } else {
        0.0
    };
    eprintln!(
        "  Done in {:.0} ms | {input_size} -> {output_size} bytes ({:.1}%)",
        elapsed.as_secs_f64() * 1000.0,
        ratio * 100.0,
    );
}
