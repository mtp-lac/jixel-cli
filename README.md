# jixel-cli - JPEG XL Encoder CLI

A Windows command-line adapter for the [jixel](https://github.com/awxkee/jixel)
JPEG XL encoder library. The library is used **unmodified** as a crates.io
dependency (`jixel 0.4.2`), and the command-line interface follows **libjxl's
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
jixel-cli.exe photo.png out.jxl --progressive_dc 1 --group_order center-first
jixel-cli.exe photo.png out.jxl --progressive_dc 1 --progressive_dc_levels 2
```

`OUTPUT` may be omitted (writes `INPUT.jxl`; cjxl requires it).

## Options

### cjxl-compatible flags

| Flag | Meaning |
|------|---------|
| `-d, --distance <0.0..25.0>` | 0 = lossless. Default 1.0 (JPEG/GIF input: 0) |
| `-q, --quality <0..100>` | maps to distance; 100 = lossless; exclusive with `-d` |
| `-e, --effort <1..10>` | default 7; maps onto jixel's six tiers: 1 UltraFast, 2–3 Fastest, 4–7 Fast, 8 Medium, 9 Slow, 10 ExtraSlow |
| `-j, --lossless_jpeg <0\|1>` | JPEG transcode control; default 1 on JPEG input |
| `--allow_jpeg_reconstruction <0\|1>` | keep/omit the JBRD reconstruction box |
| `-m, --modular <0\|1\|2>` | VarDCT / Modular / auto (2 = jixel both-arms) |
| `-p, --progressive` | progressive encoding |
| `--progressive_dc <0\|1>` | lossy VarDCT progressive DC frame; 1 = previews refine from 1:256 to 1:8 before AC (default off, lossy-only) |
| `--group_order <scanline\|center-first\|diagonals>` | group storage order for progressive rendering (default scanline; `diagonals` is jixel-only) |
| `--strip_alpha <-1\|0\|1\|2>` | alpha stripping mode (-1 = encoder chooses) |
| `--faster_decoding <0..4>` | lossless decode-speed vs density |
| `--num_threads <-1\|0\|N>` | -1 machine default, 0 = single-threaded |
| `--patches <0\|1>` | patch dictionary (default: on, auto-off with `-p`) |
| `--dots <0\|1>` | bright/dark spot coding (default: off; takes effect at effort 9-10) |
| `--intensity_target <nits>` | HDR peak luminance hint |
| `--container <0\|1>` | `0`/unset ok; `1` rejected (jixel cannot force containers) |
| `--quiet`, `-v/--verbose`, `-V/--version`, `-h/--help` | as in cjxl |

Output format (`Encoding [VarDCT, d1.000, effort: 7]` /
`Compressed to 493 bytes (0.321 bpp).`) and exit codes (0 ok, 1 error)
mirror cjxl.

### jixel-only extensions

`--lossless` (alias `-d 0`), `--fast-lossless`, `--splines`,
`--learned-rate <0\|1>`, `--arbiter <ssimulacra2\|cvvdp>`,
`--coding-transform <xyb\|ycbcr\|rgb>`,
`--progressive_dc_levels <LEVELS>`,
`--color-space <srgb\|srgb-linear\|display-p3\|bt2020-pq\|bt2020-hlg>`,
`--icc-profile <FILE>`, `--orientation <1-8>`.

`--progressive_dc_levels <LEVELS>` limits how many squeeze levels the
`--progressive_dc 1` frame keeps (finest first; each level halves the preview
resolution). Unset squeezes down to an 8x8 preview; values clamp to what the DC
image allows. It is a jixel-only knob — cjxl's `--progressive_dc` has no level
control — and it requires `--progressive_dc=1`.

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

## Further documentation

| Topic | Document |
|-------|----------|
| Effort / speed tiers, per-mode meaning, feature gating | [docs/effort.md](<docs/effort.md>) |
| Default settings (CLI and jixel library) | [docs/default-settings.md](<docs/default-settings.md>) |
| Release notes (jixel 0.4.1 & 0.4.2) | [docs/release-notes.md](<docs/release-notes.md>) |
| Supported input formats & bit depths | [docs/supported-inputs.md](<docs/supported-inputs.md>) |
| Requirements, building & testing | [docs/building.md](<docs/building.md>) |
| cjxl alignment (exhaustive flag diff) | [docs/cjxl-alignment.md](<docs/cjxl-alignment.md>) |

## License

BSD-3-Clause OR Apache-2.0 (dual-licensed like jixel). jixel is unmodified.
