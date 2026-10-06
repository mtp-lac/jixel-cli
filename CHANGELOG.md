# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
upgraded `0.3.2` -> `0.3.4`, so the following is that range: the encoder work
that merged upstream in PRs #142-#145, now inside this binary. Entries are the
upstream commit subjects.

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
feature, but neither is wired to a CLI flag yet. `dots` corresponds to cjxl's
`--dots`; `learned_rate` has no cjxl counterpart.

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