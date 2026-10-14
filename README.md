# jixel-cli - JPEG XL Encoder CLI

A Windows command-line adapter for the [jixel](https://github.com/awxkee/jixel)
JPEG XL encoder library. The library is used **unmodified** as a crates.io
dependency (`jixel 0.4.1`), and the command-line interface follows **libjxl's
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
jixel-cli.exe hdr.jpg out.jxl -e 9 --arbiter cvvdp   # CVVDP-tuned lossy encode
jixel-cli.exe wide.png out.jxl --coding-transform ycbcr
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
| `--dots <0\|1>` | bright/dark spot coding (default: off; takes effect at effort 9-10) |
| `--intensity_target <nits>` | HDR peak luminance hint |
| `--container <0\|1>` | `0`/unset ok; `1` rejected (jixel cannot force containers) |
| `--quiet`, `-v/--verbose`, `-V/--version`, `-h/--help` | as in cjxl |

Output format (`Encoding [VarDCT, d1.000, effort: 7]` /
`Compressed to 530 bytes (0.345 bpp).`) and exit codes (0 ok, 1 error)
mirror cjxl.

## jixel-only extensions

`--lossless` (alias `-d 0`), `--fast-lossless`, `--splines`,
`--learned-rate <0\|1>`, `--arbiter <ssimulacra2\|cvvdp>`,
`--coding-transform <xyb\|ycbcr\|rgb>`,
`--color-space <srgb\|srgb-linear\|display-p3\|bt2020-pq\|bt2020-hlg>`,
`--icc-profile <FILE>`, `--orientation <1-8>`.

`--learned-rate` is the switch for jixel's learned rate pricing (default on;
pass `0` to disable). It has no cjxl counterpart.

`--arbiter` selects the perceptual metric the lossy encoder tunes its
rate-distortion decisions for (new in jixel 0.4.1; default `ssimulacra2`):

| Value | Meaning |
|-------|---------|
| `ssimulacra2` | SSIMULACRA 2 (default) |
| `cvvdp` | ColorVideoVDP via `cvvdp-lite`; prefers different transform and trellis trade-offs, aimed at HDR and color-accuracy-critical content |

`--coding-transform` selects the color transform lossy frames are coded in
(default `xyb`):

| Value | Meaning |
|-------|---------|
| `xyb` | perceptual XYB (default; required for CVVDP) |
| `ycbcr` | JPEG-matrix Cb/Y/Cr, decoder-inverted |
| `rgb` | samples unchanged (R/G/B, or C/M/Y for CMYK) |

Both are **lossy-only**: they are rejected (not silently ignored) at lossless
distance, with `--fast-lossless`, and on the lossless-JPEG-transcode path. Like
`--learned-rate`, `--arbiter` only changes the output at effort 9-10
(jixel `Speed::Slow`), because the faster tiers do not run the decisions that
consult the metric.

`--fast-lossless` selects jixel's dedicated fast encoder, whose API takes only
pixels, a color space, an alpha flag and `FlMeta`. It therefore **rejects**
`-d`, `-q`, `--lossless`, `-e`, `-m`, `-p`, `--patches`, `--faster_decoding`,
`--intensity_target`, `--num_threads`, `--splines`, `--arbiter` and
`--coding-transform` instead of silently ignoring them; `--strip_alpha`,
`--color-space`, `--icc-profile` and `--orientation` do apply.

## What's new in jixel 0.4.1

This project tracks upstream `awxkee/jixel` and ships it unmodified
(jixel-cli upgraded `0.4.0` -> `0.4.1`). Upstream 0.4.1 is PR
[#157](https://github.com/awxkee/jixel/pull/157) ("CVVDP, trellis, better low
qualities").

**New public API in 0.4.1, now exposed as flags:**

- **`--arbiter <ssimulacra2|cvvdp>`** - upstream added the `Arbiter` enum and
  `EncodeConfig::arbiter` / `with_arbiter`. It selects the perceptual metric the
  lossy encoder tunes for: SSIMULACRA 2 (default) or ColorVideoVDP, a new
  `cvvdp-lite` dependency. Each arbiter is a row of tuning constants
  (`rdoq_lambda`, `qm_ss2_distance`, `rerank_downgrades`, `dct16_luma_hf`,
  `vlq_bias_rect`), and CVVDP additionally drives a closed-loop judge on the
  reconstructed XYB image. CVVDP therefore requires the XYB coding transform,
  and - like `--learned-rate` - the arbiter only bites at effort 9-10
  (verified: `-e 9 --arbiter cvvdp` and `-e 9 --arbiter ssimulacra2` produce
  different streams; at effort 1-8 they are byte-identical).
- **`--coding-transform <xyb|ycbcr|rgb>`** - `EncodeConfig::coding_transform` /
  `with_coding_transform` (available since 0.4.0, previously unexposed): chooses
  the color transform lossy frames are coded in.

**Encoder changes that come along with the bump (no flag needed):**

- **Trellis / RDOQ improvements.** The AC trellis rate price is now
  arbiter-specific (`rdoq_lambda` 1.5 for SSIMULACRA 2, 1.25 for CVVDP).
- **Better low qualities.** The very-low-quality band ramps its rectangle-merge
  bias per arbiter (`vlq_bias_rect`), and DCT16 luma weighting follows the
  arbiter.
- **CVVDP-driven decisions.** With `--arbiter cvvdp`, the closed loop uses
  ColorVideoVDP as its exact metric where SSIMULACRA 2 was used before; the
  candidate ranking marginals still come from SSIMULACRA 2.

Upstream knobs that still have no jixel-cli flag: `progressive_passes` /
`progressive_shifts` (cjxl's `--progressive_ac`, `--qprogressive_ac`,
`--progressive_dc`), `brotli_compression` (Brotli-compressed `brob` boxes),
`fill_invisible`, CMYK input (`encode_image_cmyk*` / `InkConvention`), and the
remaining tone-mapping fields `min_nits`, `relative_to_max_display`,
`linear_below`. See [docs/cjxl-alignment.md](<docs/cjxl-alignment.md>) for the
full cjxl comparison.

## Supported inputs

PNG, JPEG, GIF, BMP, TIFF, WebP, EXR, HDR… (anything the `image` crate
decodes; no PNM). Bit depths: 8/16-bit integer (L, LA, RGB, RGBA) and
32-bit float (RGB/RGBA), lossy **and** lossless.

Float lossless uses jixel's `encode_f32_lossless_rgba`, which is bit-exact but
v1-limited to finite, non-negative samples; NaN, infinity or negative values
are rejected by the library. Such files need codestream level 10 and are
emitted in container form.

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
.\run-tests.ps1                              # 80-case cjxl-syntax matrix
cargo run --release --example verify         # 47 decode checks (jxl-oxide)
```

`verify` decodes every output with **jxl-oxide** (independent pure-Rust
decoder, dev-dependency): 18 lossless streams must round-trip **bit-exact**
(including 32-bit float RGB and RGBA), lossy streams must decode with sane
PSNR. Last run: **80/80 encode cases, 47/47 verification cases**.

## License

BSD-3-Clause OR Apache-2.0 (dual-licensed like jixel). jixel is unmodified.
