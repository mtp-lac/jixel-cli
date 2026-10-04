//! jixel-cli - Command-line JPEG XL encoder using the jixel library.
//!
//! The interface mirrors libjxl's `cjxl`: same positional arguments
//! (INPUT [OUTPUT]), same flag spellings/semantics for everything jixel's
//! public API can express; see `docs/cjxl-alignment.md`.

#![forbid(unsafe_code)]

mod cli;

use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::Parser;
use image::DynamicImage;
use jixel::{
    EncodeConfig, FlMeta, JpegTranscodeConfig, Orientation, distance_from_quality,
    encode_jpeg_lossless_with_config,
};

use cli::{Args, ModularArg};

fn main() {
    let args = match Args::try_parse() {
        Ok(a) => a,
        Err(e) => {
            // cjxl: parse errors exit 1; help/version exit 0.
            let _ = e.print();
            std::process::exit(if e.use_stderr() { 1 } else { 0 });
        }
    };
    if let Err(err) = run(&args) {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

struct Coding {
    /// lossless JPEG transcode mode (cjxl's default for JPEG inputs)
    lossless_jpeg: bool,
    /// pixel-path lossless (resolved distance == 0)
    lossless: bool,
    /// resolved visual distance for the lossy pixel path
    distance: f32,
}

fn resolve_coding(args: &Args, file_bytes: &[u8]) -> Result<Coding> {
    if args.distance.is_some() && args.quality.is_some() {
        bail!("Must not set both --distance and --quality.");
    }
    if args.lossless && (args.distance.is_some() || args.quality.is_some()) {
        bail!("--lossless cannot be combined with --distance/--quality; use --distance=0.");
    }
    if args.container == Some(1) {
        bail!(
            "--container=1 is not supported: jixel emits the container format only \
             when metadata requires it (cjxl --container=0 behaviour is the default)."
        );
    }

    // jixel's fast-lossless encoder takes only color metadata + an alpha flag
    // (`encode_fast_lossless`/`encode_fast_lossless_u16`): it has no distance,
    // effort, progressive, patches, decoding-speed, tone-mapping or thread
    // controls. Reject those combinations instead of silently dropping them.
    if args.fast_lossless {
        if args.distance.is_some() || args.quality.is_some() || args.lossless {
            bail!("--fast-lossless cannot be combined with --distance, --quality or --lossless.");
        }
        for (is_set, flag) in [
            (args.progressive, "--progressive"),
            (args.splines, "--splines"),
            (args.patches.is_some(), "--patches"),
            (args.faster_decoding != 0, "--faster_decoding"),
            (args.intensity_target > 0.0, "--intensity_target"),
            (args.effort.is_some(), "--effort"),
            (args.modular.is_some(), "--modular"),
            (args.num_threads.is_some(), "--num_threads"),
        ] {
            if is_set {
                bail!(
                    "{flag} is not supported with --fast-lossless: jixel's fast-lossless \
                     encoder accepts only color metadata and an alpha channel."
                );
            }
        }
    }

    let is_jpeg = file_bytes.starts_with(&[0xFF, 0xD8]);
    let is_gif = file_bytes.starts_with(b"GIF87a") || file_bytes.starts_with(b"GIF89a");

    // cjxl: --lossless_jpeg defaults to 1 for JPEG input, forced to 0 otherwise.
    let mut lossless_jpeg = args.lossless_jpeg.map(|v| v == 1).unwrap_or(is_jpeg);
    if !is_jpeg {
        lossless_jpeg = false;
    }
    if args.fast_lossless && is_jpeg && !args.lossless_jpeg.is_some_and(|v| v == 0) {
        bail!("--fast-lossless does not apply to JPEG transcoding; pass --lossless_jpeg=0.");
    }

    // cjxl SetDistanceFromFlags, with quality == 100 mapped to 0.0 (lossless).
    let distance = if args.lossless || args.fast_lossless {
        0.0
    } else if let Some(q) = args.quality {
        if q >= 100.0 {
            0.0
        } else {
            distance_from_quality(q)
        }
    } else if let Some(d) = args.distance {
        d
    } else if is_jpeg || is_gif {
        0.0
    } else {
        1.0
    };

    if lossless_jpeg && distance != 0.0 {
        bail!(
            "Must not set non-zero distance in combination with --lossless_jpeg=1, \
             which is set by default.\nSet --lossless_jpeg=0 to recompress the JPEG as pixels."
        );
    }

    Ok(Coding {
        lossless_jpeg,
        lossless: distance == 0.0,
        distance,
    })
}

fn output_path(args: &Args) -> PathBuf {
    args.output.clone().unwrap_or_else(|| {
        let mut p = args.input.clone();
        p.set_extension("jxl");
        p
    })
}

fn run(args: &Args) -> Result<()> {
    let start = Instant::now();

    let out_path = output_path(args);
    let file_bytes = std::fs::read(&args.input)
        .with_context(|| format!("Reading image data failed: {}", args.input.display()))?;
    let coding = resolve_coding(args, &file_bytes)?;

    if !args.quiet {
        eprintln!("JPEG XL encoder jixel-cli {}", env!("CARGO_PKG_VERSION"));
    }

    // ---- cjxl JPEG lossless transcode path ----
    if coding.lossless_jpeg {
        if args.lossless_jpeg.is_none() && !args.quiet {
            eprintln!(
                "Note: Implicit-default for JPEG is lossless-transcoding. \
                 To silence this message, set --lossless_jpeg=(1|0)."
            );
        }
        if !args.quiet {
            eprintln!(
                "Encoding [JPEG, lossless transcode, effort: {}]",
                args.effort()
            );
        }
        let tcfg = JpegTranscodeConfig::default()
            .with_jpeg_reconstruction(args.allow_jpeg_reconstruction == 1)
            .with_num_threads(args.thread_count());
        let jxl_bytes = encode_jpeg_lossless_with_config(&file_bytes, &tcfg)
            .context("JPEG transcode failed")?;
        std::fs::write(&out_path, &jxl_bytes)
            .with_context(|| format!("Could not write jxl file: {}", out_path.display()))?;
        if !args.quiet {
            print_compressed(&jxl_bytes, 0, start.elapsed(), Some(args.thread_count()));
        }
        return Ok(());
    }

    let img = image::load_from_memory(&file_bytes)
        .with_context(|| format!("Getting pixel data failed: {}", args.input.display()))?;
    let pixels = (img.width() as usize) * (img.height() as usize);

    if args.verbose > 0 {
        eprintln!(
            "Read {}x{} image, {} bytes",
            img.width(),
            img.height(),
            file_bytes.len()
        );
    }

    // Alpha stripping per cjxl --strip_alpha semantics.
    let strip = match args.strip_alpha {
        0 => false,
        1 => img.has_alpha(),
        2 => img.has_alpha() && alpha_is_opaque(&img),
        // -1: strip if empty for lossy, keep for lossless
        _ => img.has_alpha() && !coding.lossless && alpha_is_opaque(&img),
    };

    let jxl_bytes = if args.fast_lossless {
        if !args.quiet {
            // No effort/thread knobs exist on this encoder; do not imply any.
            eprintln!("Encoding [Modular, lossless, fast-lossless]");
        }
        let meta = fl_meta(args)?;
        encode_fast_lossless_dynamic(&img, strip, &meta)
            .with_context(|| format!("Fast-lossless encode failed: {}", args.input.display()))?
    } else {
        let mode = match (coding.lossless, args.modular()) {
            (true, _) => "Modular",
            (false, ModularArg::Modular) => "Modular",
            _ => "VarDCT",
        };
        if !args.quiet {
            let dist = if coding.lossless {
                "lossless".to_string()
            } else {
                format!("d{:.3}", coding.distance)
            };
            eprintln!("Encoding [{mode}, {dist}, effort: {}]", args.effort());
        }
        if (args.splines || matches!(args.modular(), ModularArg::Auto | ModularArg::Modular))
            && args.effort() < 9
            && !args.lossless
            && !args.quiet
        {
            eprintln!("WARNING: splines and lossy modular arms take effect at effort 9 or 10.");
        }
        let config = encode_config(args, &coding)?;
        encode_image_dynamic(&img, &config, strip)
            .with_context(|| format!("EncodeImageJXL() failed: {}", args.input.display()))?
    };

    std::fs::write(&out_path, &jxl_bytes)
        .with_context(|| format!("Could not write jxl file: {}", out_path.display()))?;

    if !args.quiet {
        let threads = if args.fast_lossless {
            None
        } else {
            Some(args.thread_count())
        };
        print_compressed(&jxl_bytes, pixels, start.elapsed(), threads);
    }
    Ok(())
}

fn encode_config(args: &Args, coding: &Coding) -> Result<EncodeConfig> {
    let mut config = EncodeConfig::default()
        .with_lossless(coding.lossless)
        .with_speed(args.speed())
        .with_decoding_speed(args.decoding_speed())
        .with_progressive(args.progressive)
        .with_patches(args.patches.map(|p| p == 1).unwrap_or(!args.progressive))
        .with_splines(args.splines)
        .with_color_encoding(args.color_space.to_color_encoding());
    if !coding.lossless {
        config = config
            .with_distance(coding.distance)
            .with_lossy_modular(args.modular().into());
    }
    if let Some(path) = &args.icc_profile {
        let profile = std::fs::read(path)
            .with_context(|| format!("Failed to read ICC profile: {}", path.display()))?;
        config = config.with_icc_profile(profile);
    }
    if let Some(v) = args.orientation {
        config = config
            .with_orientation(Orientation::from_exif(v).context("Invalid orientation value")?);
    }
    if args.intensity_target > 0.0 {
        config = config.with_intensity_target(args.intensity_target);
    }
    Ok(config.with_num_threads(args.thread_count()))
}

/// Standard encoder path (VarDCT lossy / modular lossless) for all pixel formats.
fn encode_image_dynamic(
    img: &DynamicImage,
    config: &EncodeConfig,
    strip_alpha: bool,
) -> Result<Vec<u8>> {
    let width = img.width() as usize;
    let height = img.height() as usize;

    let bytes = match img {
        DynamicImage::ImageLuma8(buf) => {
            jixel::encode_image_gray(buf.as_raw(), width, height, config)?
        }
        DynamicImage::ImageLumaA8(buf) => {
            if strip_alpha {
                let luma: Vec<u8> = buf
                    .as_raw()
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| p[0])
                    .collect();
                jixel::encode_image_gray(&luma, width, height, config)?
            } else {
                jixel::encode_image_gray_alpha(buf.as_raw(), width, height, config)?
            }
        }
        DynamicImage::ImageRgb8(buf) => jixel::encode_image(buf.as_raw(), width, height, config)?,
        DynamicImage::ImageRgba8(buf) => {
            if strip_alpha {
                let rgb = drop_alpha(buf.as_raw());
                jixel::encode_image(&rgb, width, height, config)?
            } else {
                jixel::encode_image_with_alpha(buf.as_raw(), width, height, config)?
            }
        }
        DynamicImage::ImageLuma16(buf) => {
            jixel::encode_image_gray_16bit(buf.as_raw(), width, height, config)?
        }
        DynamicImage::ImageLumaA16(buf) => {
            if strip_alpha {
                let luma: Vec<u16> = buf
                    .as_raw()
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| p[0])
                    .collect();
                jixel::encode_image_gray_16bit(&luma, width, height, config)?
            } else {
                jixel::encode_image_gray_alpha_16bit(buf.as_raw(), width, height, config)?
            }
        }
        DynamicImage::ImageRgb16(buf) => {
            jixel::encode_image_16bit(buf.as_raw(), width, height, config)?
        }
        DynamicImage::ImageRgba16(buf) => {
            if strip_alpha {
                let rgb = drop_alpha(buf.as_raw());
                jixel::encode_image_16bit(&rgb, width, height, config)?
            } else {
                jixel::encode_image_with_alpha_16bit(buf.as_raw(), width, height, config)?
            }
        }
        // jixel encodes f32 losslessly (finite, non-negative samples only) via
        // encode_f32_lossless_rgba; the guard lives in the library.
        DynamicImage::ImageRgb32F(buf) => {
            jixel::encode_image_f32(buf.as_raw(), width, height, config)?
        }
        DynamicImage::ImageRgba32F(buf) => {
            if strip_alpha {
                let rgb = drop_alpha(buf.as_raw());
                jixel::encode_image_f32(&rgb, width, height, config)?
            } else {
                jixel::encode_image_with_alpha_f32(buf.as_raw(), width, height, config)?
            }
        }
        // DynamicImage is #[non_exhaustive]; refuse to silently re-quantize
        // any pixel format the encoder cannot handle natively.
        other => bail!("unsupported pixel format: {:?}", other.color()),
    };
    Ok(bytes)
}

/// Fast-lossless path. Only integer formats are supported by this encoder.
fn encode_fast_lossless_dynamic(
    img: &DynamicImage,
    strip_alpha: bool,
    meta: &FlMeta,
) -> Result<Vec<u8>> {
    let width = img.width() as usize;
    let height = img.height() as usize;

    let bytes = match img {
        DynamicImage::ImageLuma8(buf) => jixel::encode_fast_lossless(
            buf.as_raw(),
            width,
            height,
            jixel::ColorSpace::Gray,
            false,
            meta,
        )?,
        DynamicImage::ImageLumaA8(buf) => {
            if strip_alpha {
                let luma: Vec<u8> = buf
                    .as_raw()
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| p[0])
                    .collect();
                jixel::encode_fast_lossless(
                    &luma,
                    width,
                    height,
                    jixel::ColorSpace::Gray,
                    false,
                    meta,
                )?
            } else {
                jixel::encode_fast_lossless(
                    buf.as_raw(),
                    width,
                    height,
                    jixel::ColorSpace::Gray,
                    true,
                    meta,
                )?
            }
        }
        DynamicImage::ImageRgb8(buf) => jixel::encode_fast_lossless(
            buf.as_raw(),
            width,
            height,
            jixel::ColorSpace::Rgb,
            false,
            meta,
        )?,
        DynamicImage::ImageRgba8(buf) => {
            if strip_alpha {
                let rgb = drop_alpha(buf.as_raw());
                jixel::encode_fast_lossless(
                    &rgb,
                    width,
                    height,
                    jixel::ColorSpace::Rgb,
                    false,
                    meta,
                )?
            } else {
                jixel::encode_fast_lossless(
                    buf.as_raw(),
                    width,
                    height,
                    jixel::ColorSpace::Rgb,
                    true,
                    meta,
                )?
            }
        }
        DynamicImage::ImageLuma16(buf) => jixel::encode_fast_lossless_u16(
            buf.as_raw(),
            width,
            height,
            jixel::ColorSpace::Gray,
            false,
            16,
            meta,
        )?,
        DynamicImage::ImageLumaA16(buf) => {
            if strip_alpha {
                let luma: Vec<u16> = buf
                    .as_raw()
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| p[0])
                    .collect();
                jixel::encode_fast_lossless_u16(
                    &luma,
                    width,
                    height,
                    jixel::ColorSpace::Gray,
                    false,
                    16,
                    meta,
                )?
            } else {
                jixel::encode_fast_lossless_u16(
                    buf.as_raw(),
                    width,
                    height,
                    jixel::ColorSpace::Gray,
                    true,
                    16,
                    meta,
                )?
            }
        }
        DynamicImage::ImageRgb16(buf) => jixel::encode_fast_lossless_u16(
            buf.as_raw(),
            width,
            height,
            jixel::ColorSpace::Rgb,
            false,
            16,
            meta,
        )?,
        DynamicImage::ImageRgba16(buf) => {
            if strip_alpha {
                let rgb = drop_alpha(buf.as_raw());
                jixel::encode_fast_lossless_u16(
                    &rgb,
                    width,
                    height,
                    jixel::ColorSpace::Rgb,
                    false,
                    16,
                    meta,
                )?
            } else {
                jixel::encode_fast_lossless_u16(
                    buf.as_raw(),
                    width,
                    height,
                    jixel::ColorSpace::Rgb,
                    true,
                    16,
                    meta,
                )?
            }
        }
        DynamicImage::ImageRgb32F(_) | DynamicImage::ImageRgba32F(_) => {
            bail!("fast_lossless does not support float images; use --lossless (-d 0)");
        }
        other => bail!(
            "fast_lossless: unsupported pixel format: {:?}",
            other.color()
        ),
    };
    Ok(bytes)
}

/// True when every alpha sample is at full opacity.
fn alpha_is_opaque(img: &DynamicImage) -> bool {
    match img {
        DynamicImage::ImageRgba8(b) => b.as_raw().as_chunks::<4>().0.iter().all(|p| p[3] == 255),
        DynamicImage::ImageLumaA8(b) => b.as_raw().as_chunks::<2>().0.iter().all(|p| p[1] == 255),
        DynamicImage::ImageRgba16(b) => b.as_raw().as_chunks::<4>().0.iter().all(|p| p[3] == 65535),
        DynamicImage::ImageLumaA16(b) => {
            b.as_raw().as_chunks::<2>().0.iter().all(|p| p[1] == 65535)
        }
        DynamicImage::ImageRgba32F(b) => b.as_raw().as_chunks::<4>().0.iter().all(|p| p[3] >= 1.0),
        _ => true,
    }
}

/// Drop the 4th channel from interleaved RGBA samples (u8/u16/f32).
fn drop_alpha<T: Copy>(input: &[T]) -> Vec<T> {
    input
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|c| [c[0], c[1], c[2]])
        .collect()
}

/// cjxl-style summary line. `pixels == 0` skips the bpp/speed stats (transcode);
/// `threads == None` omits the thread line (the fast-lossless encoder is
/// single-threaded, so reporting a worker count would be misleading).
fn print_compressed(
    bytes: &[u8],
    pixels: usize,
    elapsed: std::time::Duration,
    threads: Option<usize>,
) {
    if bytes.len() < 100000 {
        eprint!("Compressed to {} bytes ", bytes.len());
    } else {
        eprint!("Compressed to {:.1} kB ", bytes.len() as f64 * 0.001);
    }
    if pixels > 0 {
        let bpp = bytes.len() as f64 * 8.0 / pixels as f64;
        eprintln!("({bpp:.3} bpp).");
        let mps = pixels as f64 / elapsed.as_secs_f64().max(1e-6) * 1e-6;
        match threads {
            Some(n) => eprintln!("Using {n} threads, average speed: {mps:.1} MP/s."),
            None => eprintln!("Average speed: {mps:.1} MP/s."),
        }
    } else {
        eprintln!();
    }
}

fn fl_meta(args: &Args) -> Result<FlMeta> {
    let mut meta = args.color_space.to_fl_meta();
    if let Some(path) = &args.icc_profile {
        meta.icc = Some(
            std::fs::read(path)
                .with_context(|| format!("Failed to read ICC profile: {}", path.display()))?,
        );
    }
    if let Some(v) = args.orientation {
        meta.orientation = Orientation::from_exif(v).context("Invalid orientation value")?;
    }
    Ok(meta)
}
