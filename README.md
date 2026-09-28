# jixel-cli - JPEG XL Encoder CLI

A Windows command-line adapter for the [jixel](https://github.com/awxkee/jixel)
JPEG XL encoder library. The library is used **unmodified** as a crates.io
dependency (`jixel 0.3`), and the command-line interface follows **libjxl's
`cjxl`**: same positionals, flag spellings, and default semantics wherever
jixel's API can express them. See [docs/cjxl-alignment.md](<docs/cjxl-alignment.md>)
for the full diff.

## Usage

```
jixel-cli [OPTIONS] INPUT [OUTPUT]
```

```powershell
# cjxl-style invocations
jixel-cli.exe photo.png photo.jxl              # visually lossless (d=1.0)
jixel-cli.exe photo.png photo.jxl -d 0         # mathematically lossless
jixel-cli.exe photo.png photo.jxl -q 85        # JPEG-style quality
jixel-cli.exe photo.jpg out.jxl                # automatic lossless JPEG transcode
jixel-cli.exe photo.jpg out.jxl -j 0 -d 1.5    # recompress JPEG pixels lossily
jixel-cli.exe big.png out.jxl -e 9 --splines   # maximum density + splines
jixel-cli.exe art.png out.jxl -m 1             # lossy modular arm
jixel-cli.exe photo.png out.jxl --fast-lossless
jixel-cli.exe icon.png out.jxl --strip_alpha 2 --num_threads 4
```

`OUTPUT` may be omitted (writes `INPUT.jxl`; cjxl requires it).

## cjxl-compatible flags

| Flag | Meaning |
|------|---------|
| `-d, --distance <0.0..25.0>` | 0 = lossless. Default 1.0 (JPEG/GIF input: 0) |
| `-q, --quality <0..100>` | maps to distance; 100 = lossless; exclusive with `-d` |
| `-e, --effort <1..10>` | default 7; jixel has 3 tiers (1–3 / 4–8 / 9–10) |
| `-j, --lossless_jpeg <0\|1>` | JPEG transcode control; default 1 on JPEG input |
| `--allow_jpeg_reconstruction <0\|1>` | keep/omit the JBRD reconstruction box |
| `-m, --modular <0\|1\|2>` | VarDCT / Modular / auto (2 = jixel both-arms) |
| `-p, --progressive` | progressive encoding |
| `--strip_alpha <-1\|0\|1\|2>` | alpha stripping mode (-1 = encoder chooses) |
| `--faster_decoding <0..4>` | lossless decode-speed vs density |
| `--num_threads <-1\|0\|N>` | -1 machine default, 0 = single-threaded |
| `--patches <0\|1>` | patch dictionary (default: on, auto-off with `-p`) |
| `--intensity_target <nits>` | HDR peak luminance hint |
| `--container <0\|1>` | `0`/unset ok; `1` rejected (jixel cannot force containers) |
| `--quiet`, `-v/--verbose`, `-V/--version`, `-h/--help` | as in cjxl |

Output format (`Encoding [VarDCT, d1.000, effort: 7]` /
`Compressed to 530 bytes (0.345 bpp).`) and exit codes (0 ok, 1 error)
mirror cjxl.

## jixel-only extensions

`--lossless` (alias `-d 0`), `--fast-lossless`, `--splines`,
`--color-space <srgb|srgb-linear|display-p3|bt2020-pq|bt2020-hlg>`,
`--icc-profile <FILE>`, `--orientation <1-8>`.

## Supported inputs

PNG, JPEG, GIF, BMP, TIFF, WebP, EXR, HDR… (anything the `image` crate
decodes; no PNM). Bit depths: 8/16-bit integer (L, LA, RGB, RGBA) and
32-bit float (RGB/RGBA, lossy only).

## Requirements

- **Rust 1.94+**, `x86_64-pc-windows-msvc` toolchain
- **Visual Studio Build Tools** (C++ workload) for the MSVC linker
- Network on first build (crates.io)

## Building & testing

```powershell
cd jixel-cli
cargo build --release        # -> target\release\jixel-cli.exe
# or full pipeline:
.\build.ps1 -Test
```

Test rig:

```powershell
.\gen-test-images.ps1                        # 8-bit PNG/JPEG fixtures
cargo run --release --example gen16          # 16-bit/float fixtures
.\run-tests.ps1                              # 45-case cjxl-syntax matrix
cargo run --release --example verify         # 38 decode checks (jxl-oxide)
```

`verify` decodes every output with **jxl-oxide** (independent pure-Rust
decoder, dev-dependency): 16 lossless streams must round-trip **bit-exact**,
lossy streams must decode with sane PSNR. Last run: **45/45 encode cases,
38/38 verification cases**.

## License

BSD-3-Clause OR Apache-2.0 (dual-licensed like jixel). jixel is unmodified.
