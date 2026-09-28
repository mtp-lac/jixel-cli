//! CLI arguments for jixel-cli, aligned with libjxl's `cjxl`.
//!
//! Long/short flag spellings and value semantics follow `cjxl` (main branch)
//! wherever jixel's API can express them. jixel-only extensions are marked.

use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueEnum};
use jixel::{ColorEncoding, DecodingSpeed, FlMeta, LossyModular, Speed};

/// Parse a bounded float flag value with a cjxl-style message.
fn bounded_f32(min: f32, max: f32) -> impl Fn(&str) -> Result<f32, String> + Clone {
    move |s: &str| {
        let v: f32 = s.parse().map_err(|e| format!("invalid float: {e}"))?;
        if v.is_finite() && (min..=max).contains(&v) {
            Ok(v)
        } else {
            Err(format!("{v} is not in {min}..={max}"))
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "jixel-cli",
    version,
    about = "JPEG XL encoder (cjxl-compatible CLI on top of jixel)"
)]
pub struct Args {
    /// the input image (PNG, JPEG, GIF, PNM, WebP, EXR, ...)
    pub input: PathBuf,

    /// the compressed JXL output file (default: INPUT with .jxl)
    #[arg(value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    /// Target visual distance in JND units, 0.0 .. 25.0.
    ///   0.0 = lossless (default for JPEG/GIF input).
    ///   1.0 = visually lossless (default otherwise).
    #[arg(
        short,
        long,
        value_name = "DISTANCE",
        value_parser = bounded_f32(0.0, 25.0)
    )]
    pub distance: Option<f32>,

    /// Quality 0 .. 100, mapped to --distance (100 = lossless).
    /// Mutually exclusive with --distance.
    #[arg(
        short,
        long,
        value_name = "QUALITY",
        value_parser = bounded_f32(0.0, 100.0)
    )]
    pub quality: Option<f32>,

    /// Encoder effort, 1 .. 10 (default 7). Higher = smaller output at the
    /// same quality. Mapped onto jixel's three speed tiers.
    #[arg(
        short,
        long,
        value_name = "EFFORT",
        default_value_t = 7,
        value_parser = clap::value_parser!(u32).range(1..=10)
    )]
    pub effort: u32,

    /// 0 = decode JPEG input to pixels and reencode; 1 = losslessly transcode
    /// JPEG data (default 1 when the input is a JPEG).
    #[arg(
        short = 'j',
        long = "lossless_jpeg",
        value_name = "0|1",
        value_parser = clap::value_parser!(u32).range(0..=1)
    )]
    pub lossless_jpeg: Option<u32>,

    /// Disable/enable storing JPEG reconstruction metadata with lossless JPEG
    /// transcoding (default 1).
    #[arg(
        long = "allow_jpeg_reconstruction",
        value_name = "0|1",
        default_value_t = 1,
        value_parser = clap::value_parser!(u32).range(0..=1)
    )]
    pub allow_jpeg_reconstruction: u32,

    /// 0 = use VarDCT mode, 1 = use Modular mode, 2 = heuristic "auto"
    /// (jixel both-arms extension). Lossy only.
    #[arg(short, long, value_name = "MODULAR", value_enum, default_value = "0")]
    pub modular: ModularArg,

    /// More progressive/responsive decoding.
    #[arg(short, long)]
    pub progressive: bool,

    /// Alpha stripping mode:
    ///   -1 = encoder chooses (strip if empty for lossy, keep for lossless)
    ///    0 = never strip.  1 = always strip.  2 = strip if fully opaque.
    #[arg(
        long = "strip_alpha",
        value_name = "-1|0|1|2",
        default_value_t = -1,
        value_parser = clap::value_parser!(i8).range(-1..=2)
    )]
    pub strip_alpha: i8,

    /// Higher values improve decode speed at the expense of quality or
    /// density, 0 .. 4 (default 0). Lossless only.
    #[arg(
        long = "faster_decoding",
        value_name = "0..4",
        default_value_t = 0,
        value_parser = clap::value_parser!(u32).range(0..=4)
    )]
    pub faster_decoding: u32,

    /// Number of worker threads (default -1).
    ///   -1 = machine default.  0 = do not use multithreading.
    #[arg(
        long = "num_threads",
        value_name = "THREADS",
        default_value_t = -1,
        value_parser = clap::value_parser!(i64).range(-1..)
    )]
    pub num_threads: i64,

    /// 0 = do not use the container format unless it is needed (jixel's only
    /// mode). 1 is rejected: jixel cannot force a container.
    #[arg(long, value_name = "0|1", value_parser = clap::value_parser!(u32).range(0..=1))]
    pub container: Option<u32>,

    /// 0 = disable patches, 1 = enable. Default = encoder chooses
    /// (enabled, but disabled automatically with --progressive).
    #[arg(long, value_name = "0|1", value_parser = clap::value_parser!(u32).range(0..=1))]
    pub patches: Option<u32>,

    /// Upper bound on the intensity level present in the image, in nits.
    /// 0 = choose a sensible value based on the color encoding (default).
    #[arg(
        long = "intensity_target",
        value_name = "NITS",
        default_value_t = 0.0,
        value_parser = bounded_f32(0.0, f32::INFINITY)
    )]
    pub intensity_target: f32,

    /// Minimal printing.
    #[arg(long)]
    pub quiet: bool,

    /// Verbose printing; can be repeated.
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    // ---------- jixel-only extensions (no cjxl equivalent) ----------
    /// Lossless encoding, alias for --distance=0 (pixel path).
    #[arg(long)]
    pub lossless: bool,

    /// jixel fast-lossless encoder (8/16-bit integer images only).
    #[arg(long)]
    pub fast_lossless: bool,

    /// Enable experimental spline detection (jixel; needs effort >= 9).
    #[arg(long)]
    pub splines: bool,

    /// Color encoding written into the codestream (jixel presets).
    #[arg(long, value_enum, default_value = "srgb")]
    pub color_space: ColorSpaceArg,

    /// Embed an ICC profile from file (forces the container form).
    #[arg(long, value_name = "ICC_FILE")]
    pub icc_profile: Option<PathBuf>,

    /// EXIF orientation value (1-8) to signal for display.
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=8))]
    pub orientation: Option<u8>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ModularArg {
    /// VarDCT mode (cjxl `-m 0`).
    #[value(alias = "0")]
    Vardct,
    /// Modular mode (cjxl `-m 1`).
    #[value(alias = "1")]
    Modular,
    /// Heuristic: encode both arms, keep the smaller (jixel `Auto`).
    #[value(alias = "2", alias = "auto")]
    Auto,
}

impl From<ModularArg> for LossyModular {
    fn from(v: ModularArg) -> Self {
        match v {
            ModularArg::Vardct => LossyModular::Off,
            ModularArg::Modular => LossyModular::Force,
            ModularArg::Auto => LossyModular::Auto,
        }
    }
}

impl Args {
    /// Resolve the cjxl effort range onto jixel's three speed tiers.
    pub fn speed(&self) -> Speed {
        match self.effort {
            1..=3 => Speed::Fastest,
            4..=8 => Speed::Fast,
            _ => Speed::Slow,
        }
    }

    /// Resolve cjxl `--faster_decoding` 0..4 onto jixel's DecodingSpeed.
    pub fn decoding_speed(&self) -> DecodingSpeed {
        match self.faster_decoding {
            0 => DecodingSpeed::Slow,
            1..=2 => DecodingSpeed::Fast,
            _ => DecodingSpeed::Fastest,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorSpaceArg {
    /// sRGB primaries + sRGB transfer (default)
    Srgb,
    /// sRGB primaries, linear transfer
    SrgbLinear,
    /// Display P3 primaries + sRGB transfer
    DisplayP3,
    /// BT.2020 primaries + PQ transfer (Rec.2100 HDR10)
    Bt2020Pq,
    /// BT.2020 primaries + HLG transfer
    Bt2020Hlg,
}

impl ColorSpaceArg {
    pub fn to_color_encoding(self) -> ColorEncoding {
        match self {
            ColorSpaceArg::Srgb => ColorEncoding::srgb(),
            ColorSpaceArg::SrgbLinear => ColorEncoding::srgb_linear(),
            ColorSpaceArg::DisplayP3 => ColorEncoding::display_p3(),
            ColorSpaceArg::Bt2020Pq => ColorEncoding::bt2020_pq(),
            ColorSpaceArg::Bt2020Hlg => ColorEncoding::bt2020_hlg(),
        }
    }

    pub fn to_fl_meta(self) -> FlMeta {
        match self {
            ColorSpaceArg::Srgb => FlMeta::srgb(),
            ColorSpaceArg::SrgbLinear => FlMeta::linear(),
            ColorSpaceArg::DisplayP3 => FlMeta::display_p3(),
            ColorSpaceArg::Bt2020Pq => FlMeta::rec2100_pq(),
            ColorSpaceArg::Bt2020Hlg => FlMeta::rec2100_hlg(),
        }
    }
}
