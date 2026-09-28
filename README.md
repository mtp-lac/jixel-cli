# jixel-cli - JPEG XL Encoder CLI

A Windows command-line adapter for the [jixel](https://github.com/awxkee/jixel)
JPEG XL encoder library. The library is used **unmodified** as a crates.io
dependency (`jixel 0.3`).

## Features

- **Lossy (VarDCT)**, **lossless (Modular)**, **fast-lossless**, and **JPEG
  lossless transcode** modes
- **Color encodings** supported by jixel: sRGB, sRGB-linear, Display P3,
  BT.2020-PQ, BT.2020-HLG (grayscale is selected automatically from the input)
- **Bit depths**: 8-bit, 16-bit integer and 32-bit float inputs;
  alpha (RGBA, gray+alpha) handled natively
- **Quality/speed tradeoffs**: `--speed fastest|fast|slow`,
  `--decoding-speed`, `--progressive`, `--lossy-modular auto|force`,
  `--splines` (jixel's experimental spline arm)
- **Multi-threaded** lossy/lossless encoding (`-j`, default = auto)
- ICC profile embedding and EXIF orientation signaling
- `--no-alpha` to drop an existing alpha channel

## Requirements

- **Rust 1.94+** with the `x86_64-pc-windows-msvc` toolchain
- **Visual Studio Build Tools** (C++ workload) for the MSVC linker
- Network access on first build (crates.io downloads)

## Building

```powershell
cd jixel-cli
cargo build --release        # -> target\release\jixel-cli.exe
# or the helper script:
.\build.ps1                  # build + size report
.\build.ps1 -Test            # build + generate images + encode matrix + decode verify
```

## Testing

The repo includes a full test rig used to validate the adapter:

```powershell
.\gen-test-images.ps1                          # 8-bit PNG/JPEG (GDI+)
cargo run --release --example gen16            # 16-bit PNGs + float EXR
.\run-tests.ps1                                # 30-case encode matrix + signature checks
cargo run --release --example verify           # decodes every output with jxl-oxide
```

`verify` decodes the encoder's output with **jxl-oxide** (an independent
pure-Rust JPEG XL decoder, dev-dependency only) and asserts:

- lossless / fast-lossless streams round-trip **bit-exact** (RGB, RGBA,
  gray, gray+alpha, 16-bit);
- lossy streams decode cleanly, reporting PSNR/max error;
- JPEG-transcoded streams match the original JPEG pixels.

Last run: **30/30 encode cases OK, 27/27 decode-verification cases passed**
(lossless bit-exact; q90 ≈ 47-52 dB PSNR; q30 ≈ 37 dB).

## Usage

```powershell
# Basic lossy encoding (quality 90 by default)
jixel-cli.exe input.png -o output.jxl

# Lossless
jixel-cli.exe input.png --lossless

# Fast-lossless (8/16-bit integer inputs only)
jixel-cli.exe input.png --fast-lossless

# JPEG lossless transcode (bit-exact; input must be a real JPEG)
jixel-cli.exe photo.jpg --jpeg-lossless

# Slower, denser lossy encode with splines enabled
jixel-cli.exe art.png -s slow --splines

# Drop alpha, 4 threads, quiet
jixel-cli.exe input.png --no-alpha -j 4 --quiet
```

## Options

| Option | Description |
|--------|-------------|
| `<INPUT>` | Input image (PNG, JPEG, BMP, TIFF, WebP, GIF, EXR, ...) |
| `-o, --output <OUT>` | Output .jxl path (default: input with `.jxl`) |
| `-q, --quality <1-100>` | Lossy quality (default 90; ignored when lossless) |
| `--lossless` | Lossless modular encoding |
| `--fast-lossless` | jixel fast-lossless encoder (integers only; single-threaded) |
| `-s, --speed <S>` | `fastest` \| `fast` (default) \| `slow` |
| `--decoding-speed <S>` | `fastest` \| `fast` \| `slow` (default; lossless only) |
| `--color-space <CS>` | `srgb` (default) \| `srgb-linear` \| `display-p3` \| `bt2020-pq` \| `bt2020-hlg` |
| `--no-alpha` | Drop the alpha channel |
| `--progressive` | Progressive encoding |
| `--no-patches` | Disable the patch dictionary (patches on by default) |
| `--splines` | Enable spline detection (requires `--speed slow`) |
| `--lossy-modular <M>` | `off` \| `auto` \| `force` (requires `--speed slow`) |
| `-j, --threads <N>` | Worker threads, 0 = auto (default) |
| `--icc-profile <FILE>` | Embed an ICC profile (output becomes a container) |
| `--orientation <1-8>` | Signal EXIF orientation for display |
| `--jpeg-lossless` | Transcode JPEG input losslessly (keeps reconstruction) |
| `--quiet` | Suppress progress |
| `--verbose` | Extra diagnostics |

## Output format

Bare JXL codestream (`FF 0A ...`) by default. When metadata forces a container
(16-bit alpha, embedded ICC/EXIF, JPEG reconstruction), the ISOBMFF container
signature (`00 00 00 0C 'JXL ' 0D 0A 87 0A`) is emitted instead. Both are
valid JPEG XL.

## License

BSD-3-Clause OR Apache-2.0 (same dual license as jixel). jixel itself is
unmodified; this project only adapts its public API to a CLI.
