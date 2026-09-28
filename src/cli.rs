//! CLI argument definitions for jixel-cli

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use jixel::{ColorEncoding, DecodingSpeed, FlMeta, LossyModular, Speed};

#[derive(Parser, Debug)]
#[command(
    name = "jixel-cli",
    version,
    about = "JPEG XL encoder CLI built on the jixel library"
)]
pub struct Args {
    /// Input image file (PNG, JPEG, BMP, TIFF, WebP, etc.)
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// Output .jxl file (default: input with .jxl extension)
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    /// Quality 1-100 for lossy encoding (ignored for lossless)
    #[arg(
        short,
        long,
        default_value = "90",
        value_parser = clap::value_parser!(u32).range(1..=100)
    )]
    pub quality: u32,

    /// Lossless encoding via the modular encoder
    #[arg(long)]
    pub lossless: bool,

    /// Fast-lossless encoder (integer images only; ignores --speed/--threads)
    #[arg(long, conflicts_with = "lossless")]
    pub fast_lossless: bool,

    /// Encoder speed/quality tradeoff
    #[arg(short, long, value_enum, default_value = "fast")]
    pub speed: SpeedArg,

    /// Decode-speed/density tradeoff (lossless only)
    #[arg(long, value_enum, default_value = "slow")]
    pub decoding_speed: DecodingSpeedArg,

    /// Color encoding written into the codestream
    #[arg(long, value_enum, default_value = "srgb")]
    pub color_space: ColorSpaceArg,

    /// Drop the alpha channel even if the input has one
    #[arg(long)]
    pub no_alpha: bool,

    /// Progressive encoding
    #[arg(long)]
    pub progressive: bool,

    /// Disable the patch dictionary (enabled by default)
    #[arg(long)]
    pub no_patches: bool,

    /// Enable experimental spline detection (Slow speed only)
    #[arg(long)]
    pub splines: bool,

    /// Lossy modular arm selection (Slow speed only)
    #[arg(long, value_enum)]
    pub lossy_modular: Option<LossyModularArg>,

    /// Thread count (0 = auto-detect)
    #[arg(short = 'j', long, default_value = "0")]
    pub threads: usize,

    /// Embed an ICC profile from file (forces container output)
    #[arg(long, value_name = "ICC_FILE")]
    pub icc_profile: Option<PathBuf>,

    /// EXIF orientation value (1-8) to signal for display
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=8))]
    pub orientation: Option<u8>,

    /// Losslessly transcode a JPEG file (input must be a real JPEG)
    #[arg(long)]
    pub jpeg_lossless: bool,

    /// Suppress progress output
    #[arg(long)]
    pub quiet: bool,

    /// Extra diagnostics
    #[arg(long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SpeedArg {
    /// No transform search: plain 8x8 DCT blocks
    Fastest,
    /// Balanced (default)
    Fast,
    /// Full transform search; required for splines/lossy-modular
    Slow,
}

impl From<SpeedArg> for Speed {
    fn from(v: SpeedArg) -> Self {
        match v {
            SpeedArg::Fastest => Speed::Fastest,
            SpeedArg::Fast => Speed::Fast,
            SpeedArg::Slow => Speed::Slow,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DecodingSpeedArg {
    /// No weighted predictor, no meta-adaptive trees
    Fastest,
    /// No weighted predictor
    Fast,
    /// All coding tools, densest output (default)
    Slow,
}

impl From<DecodingSpeedArg> for DecodingSpeed {
    fn from(v: DecodingSpeedArg) -> Self {
        match v {
            DecodingSpeedArg::Fastest => DecodingSpeed::Fastest,
            DecodingSpeedArg::Fast => DecodingSpeed::Fast,
            DecodingSpeedArg::Slow => DecodingSpeed::Slow,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LossyModularArg {
    /// VarDCT only (default)
    Off,
    /// Encode both arms, keep the smaller (Slow speed)
    Auto,
    /// Force the modular arm where supported (Slow speed)
    Force,
}

impl From<LossyModularArg> for LossyModular {
    fn from(v: LossyModularArg) -> Self {
        match v {
            LossyModularArg::Off => LossyModular::Off,
            LossyModularArg::Auto => LossyModular::Auto,
            LossyModularArg::Force => LossyModular::Force,
        }
    }
}
