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

### Fixed
- Stricter parameter validation for `--fast-lossless` encoder (rejects incompatible flags instead of silently ignoring)
- Changed `effort`, `modular`, and `num_threads` from required args with defaults to optional args with proper method accessors

## Upstream Jixel Changelog

### v0.3.4 (2026-10-04)
- ICC compression improvements
- Refined splines and patches behavior
- Learned rate pricing optimizations

### v0.3.3 (2026-09-30)
- Fine blocks fixes
- Learned rate prices
- Splines improvements

### v0.3.2 (2026-09-22)
- Fix patches and splines behavior

### v0.3.1 (2026-09-21)
- Patches and splines features

### v0.3.0 (2026-09-20)
- Initial 0.3.x series release
- API breaking changes from 0.2.x series