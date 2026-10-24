# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-10-24

### Changed
- Updated jixel dependency from 0.4.2 to 0.5.0 (latest upstream release)
- Public API unchanged from 0.4.2: same flags and defaults; the bump ships
  upstream's internal encoder improvements (new DCT 32x8 / 8x32 transform
  selection, improved ink layers and region frames, a dequant-bias fix, and
  general speed/density work), so encoded output may differ from 0.4.2
- Upstream moved the perceptual-metric crates (`cvvdp-lite`,
  `ssimu2-lite`) behind a new `extraslow` Cargo feature; it is part of
  jixel's default features, so `--arbiter ssimulacra2|cvvdp` behaviour is
  unchanged
- Refreshed README, docs/release-notes.md and docs/cjxl-alignment.md for
  jixel 0.5.0

## [0.4.2] - 2026-10-20

### Changed
- Updated jixel dependency from 0.4.1 to 0.4.2 (latest upstream release)
- Documented jixel's default settings (`EncodeConfig::default`, `JpegTranscodeConfig`,
  `FlMeta`) and the effective CLI defaults in docs/default-settings.md
- Moved detailed reference documentation out of README into `docs/` (effort tiers,
  default settings, release notes, supported inputs, building & testing); README
  now keeps basic usage and options and links to the individual documents
- Refreshed README and docs/cjxl-alignment.md for jixel 0.4.2; corrected the list of
  upstream knobs that still have no CLI flag (`--progressive_dc` is now exposed)

### Added
- `--progressive_dc <0|1>` flag exposing jixel 0.4.2's `EncodeConfig::progressive_dc`
  (lossy VarDCT progressive DC frame; default off)
- `--progressive_dc_levels <LEVELS>` jixel-only flag exposing
  `EncodeConfig::progressive_dc_levels` (limits the progressive DC frame's squeeze
  levels; requires `--progressive_dc=1`)
- `--group_order <scanline|center-first|diagonals>` flag exposing jixel 0.4.2's
  `GroupOrder` enum (`scanline`/`center-first` match cjxl `--group_order=0/1`;
  `diagonals` is a jixel extension)
- Encode-matrix cases for the new flags and their rejection rules, plus jxl-oxide
  decode checks for the new outputs
- Validation that rejects the new lossy-only controls where they cannot take effect
  (`--lossless`, `-d 0`, `-q 100`, `--fast-lossless`, and the lossless-JPEG-transcode
  path)
