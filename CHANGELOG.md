# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2026-10-14

### Changed
- Updated jixel dependency from 0.4.0 to 0.4.1 (latest upstream release)
- `-e/--effort` now resolves onto all six jixel speed tiers instead of three
  (`UltraFast`, `Fastest`, `Fast`, `Medium`, `Slow`, `ExtraSlow`): effort 1
  UltraFast, 2-3 Fastest, 4-7 Fast, 8 Medium, 9 Slow, 10 ExtraSlow. The mapping
  is monotonic and default-preserving (effort 7 stays on jixel's `#[default]`
  `Speed::Fast`), so only `-e 1`, `-e 8` and `-e 10` change output
- Refreshed README for jixel 0.4.1 and corrected the list of upstream knobs
  that still have no CLI flag

### Added
- `--arbiter <ssimulacra2|cvvdp>` flag exposing jixel 0.4.1's new `Arbiter`
  enum / `EncodeConfig::arbiter`: selects the perceptual metric the lossy
  encoder tunes rate-distortion decisions for (SSIMULACRA 2 default, or
  ColorVideoVDP via the new `cvvdp-lite` dependency)
- `--coding-transform <xyb|ycbcr|rgb>` flag exposing
  `EncodeConfig::coding_transform`: selects the color transform lossy frames
  are coded in
- Encode-matrix cases for the new flags and their rejection rules, plus
  jxl-oxide decode checks for the new outputs (86 encode cases, 53 verification
  cases)
- Effort-tier boundary cases (`-e 1`, `-e 8`, `-e 10`) for both lossy and
  lossless, the latter asserting bit-exact round-trips at the new tiers
- Validation that rejects `--arbiter` / `--coding-transform` where they cannot
  take effect (`--lossless`, `-d 0`, `-q 100`, `--fast-lossless`, and the
  lossless-JPEG-transcode path)

### Fixed
- Documented that `--arbiter` only changes the output at effort 9-10
  (jixel `Speed::Slow`); at efforts 1-8 the streams are byte-identical

## [0.1.0] - 2026-10-04

### Changed
- Updated jixel dependency from 0.3 to 0.3.4 (latest upstream release)
- Updated Cargo.lock with new jixel checksum

### Added
- Full support for 32-bit float (EXR/HDR) lossless encoding via `encode_f32_lossless_rgba`
- Better documentation for `--fast-lossless` encoder restrictions
- Test images for 32-bit float RGBA encoding
- Verification cases for 32-bit float lossless round-tripping
- GitHub Actions release workflow for automated builds and releases
- `--dots <0|1>` flag (cjxl convention) exposing jixel 0.3.4's bright/dark dot
  coding via `EncodeConfig::dots`
- `--learned-rate <0|1>` jixel-only flag exposing `EncodeConfig::learned_rate`
  (learned rate pricing, default on)
- Documentation of the upstream jixel 0.3.4 encoder changes (dots coding,
  learned rate pricing, DC smoothing, splines/patches fixes) and of the
  upstream knobs that still have no CLI flag

### Fixed
- Stricter parameter validation for `--fast-lossless` encoder (rejects incompatible flags instead of silently ignoring)
- Changed `effort`, `modular`, and `num_threads` from required args with defaults to optional args with proper method accessors

## Upstream Jixel Changelog

jixel-cli pins upstream `awxkee/jixel` and ships it unmodified. This project
upgraded `0.3.4` -> `0.4.1`, so the following is that range: the encoder work
that merged upstream in PRs #146-#157, now inside this binary. Entries are the
upstream commit/PR subjects.

### v0.4.1 (2026-10-13)

Upstream PR [#157](https://github.com/awxkee/jixel/pull/157) ("CVVDP, trellis,
better low qualities"):

**New public API (0.4.0 -> 0.4.1):**

* `Arbiter` enum (`Arbiter::Ssimulacra2`, `Arbiter::Cvvdp`) +
  `EncodeConfig::arbiter` / `with_arbiter(Arbiter)` — the perceptual metric the
  **lossy** encoder tunes its rate-distortion decisions for; ignored for
  lossless. Everything metric-dependent reads a row of the new `Tuning` table
  (`rdoq_lambda`, `qm_ss2_distance`, `rerank_downgrades`, `dct16_luma_hf`,
  `vlq_bias_rect`). `Cvvdp` additionally drives a closed-loop judge on the
  reconstructed XYB image, so it needs the XYB coding transform. This adds the
  new `cvvdp-lite` dependency. Wired to `--arbiter`.

**Encoder changes:**

* Trellis / RDOQ rate pricing is now arbiter-specific
* Better low qualities (per-arbiter very-low-quality merge ramp and DCT16 luma)
* CVVDP arbiter introduced (exact closed-loop metric; candidate ranking still
  uses SSIMULACRA2 marginals)

### v0.4.0 (2026-10-12)

Upstream PRs #146-#156:

* ICC compression
* Better transcoding
* Better scale, optimizations
* Patches improvement
* Additional speed tiers (`UltraFast`, `Medium`, `ExtraSlow` — jixel now has six
  `Speed` variants instead of three)
* AQ, DCT-128, presets, EPF, matrices
* CMYK VarDCT support (`encode_image_cmyk` / `encode_image_cmyk_16bit`,
  `InkConvention`)
* `CodingTransform` (`Xyb` / `YCbCr` / `Rgb`) + `with_coding_transform`
* EXIF / XMP / gain-map metadata embedding and `BrotliCompression`

### v0.3.4 (2026-10-04)

**New public API (verified by diffing the `EncodeConfig` fields and builder
methods of 0.3.2 against 0.3.4 — the only two additions in that range):**

* `EncodeConfig::dots` + `with_dots(bool)` — experimental **bright/dark dot
  coding**: isolated bright or dark spots are coded as signed Gaussian dots
  drawn from a small template atlas, next to the lossy VarDCT frame. Dots,
  templates and the whole set pass rate-distortion tests; unprofitable sets
  fall back to VarDCT. Requires the `splines` Cargo feature. Slow speed and the
  default decoding speed only; costs encode time.
* `EncodeConfig::learned_rate` + `with_learned_rate(bool)` — toggle for **learned
  rate pricing**. Default `true`; takes effect at `Speed::Slow` only.

Both new fields are reachable from jixel-cli, which enables the `splines` Cargo
feature, and both are wired to CLI flags (`--dots`, `--learned-rate`). `dots`
corresponds to cjxl's `--dots`; `learned_rate` has no cjxl counterpart.

**Encoder changes** (upstream PRs #143, #144, #145):

* DC smoothing improvements
* DC coding
* Adding dots
* Fixing spline/dot selection, better dots, X deadzone

### v0.3.3 (2026-09-30)

Upstream PR #142 ("rate learning, block fixes"):

* Fine blocks fixes
* Learned rate prices
* Splines improvements
* DC smoothing improvements

### v0.3.2 (2026-09-22)

Upstream PR #141: fix patches and splines behavior.

### v0.3.1 (2026-09-21)

* Patches and splines features

### v0.3.0 (2026-09-20)

* Initial 0.3.x series release
* API breaking changes from 0.2.x series