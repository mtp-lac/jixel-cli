# cjxl interface alignment

Review of `jixel-cli` against **libjxl's `cjxl`** (main branch,
`tools/cjxl_main.cc`), 2026-09. Goal: same invocation syntax and flag
semantics for everything the jixel 0.3 public API can express. jixel-only
extensions are listed separately; flags jixel cannot implement are *not*
silently accepted.

```
cjxl       [OPTIONS] INPUT OUTPUT
jixel-cli  [OPTIONS] INPUT [OUTPUT]
```

## Positional arguments

| cjxl | jixel-cli | Notes |
|------|-----------|-------|
| `INPUT` (required) | `INPUT` (required) | ✓ |
| `OUTPUT` (required) | `OUTPUT` (optional) | jixel-cli derives `INPUT.jxl` when omitted (intentional superset; every cjxl invocation still works) |

## Flags implemented with cjxl semantics

| Flag | cjxl meaning | jixel-cli status |
|------|--------------|------------------|
| `-d, --distance <0.0..25.0>` | 0 = lossless; unset default: 1.0 (JPEG/GIF input: 0) | ✓ full parity, incl. the JPEG/GIF default |
| `-q, --quality <0..100>` | mapped to distance; 100 = lossless; exclusive with `-d` | ✓; q≥100 routes to the lossless path (jixel's *internal* curve ends at d=0.05 for q100 — the CLI enforces libjxl's q100=lossless contract) |
| `-e, --effort <1..10>` (default 7) | 10 discrete speed tiers | ✓ coarse-mapped onto jixel's 3 tiers: 1–3 Fastest, 4–8 Fast, 9–10 Slow. Default effort 7 = jixel `Fast`. `--allow_expert_options`/effort 11 unsupported |
| `-V, --version` | print version, exit 0 | ✓ |
| `--quiet` (no short!) | minimal printing | ✓ |
| `-v, --verbose` (repeatable) | extra info | ✓ (counted) |
| `--strip_alpha <-1|0|1|2>` | -1 encoder chooses (lossy: strip if empty; lossless: keep), 0 keep, 1 strip, 2 strip-if-opaque | ✓ incl. the empty-alpha scan |
| `-p, --progressive` | flag | ✓; like cjxl it also disables `patches` when unspecified |
| `-m, --modular <0|1>` | 0 VarDCT, 1 Modular (lossy) | ✓; extension value `2`/`auto` = jixel's both-arms heuristic |
| `-j, --lossless_jpeg <0|1>` (default 1 for JPEG magic) | implicit JPEG transcode; forced off for non-JPEG; error if combined with non-zero distance | ✓ identical auto-detect (FFD8 magic), identical note "Implicit-default for JPEG is lossless-transcoding...", identical conflict error |
| `--allow_jpeg_reconstruction <0|1>` | store/drop JBRD reconstruction box | ✓ (dropping yields a bare codestream, like cjxl) |
| `--num_threads <-1|0|N>` | -1 machine default, 0 no MT | ✓ (0 maps to jixel 1 thread) |
| `--faster_decoding <0..4>` | decode-speed vs density | ✓ mapped to jixel's 3 levels: 0→Slow, 1–2→Fast, 3–4→Fastest (lossless only, same as jixel) |
| `--patches <0|1>` | encoder chooses default | ✓ default = enabled, auto-disabled with `--progressive` (cjxl rule) |
| `--intensity_target <nits>` | 0 = auto | ✓ |
| `--container <0|1>` | 1 forces container | accepts `0`/unset only; `1` **errors**: jixel has no force-container API (metadata auto-switches to container exactly like cjxl's forced promotion) |
| Exit codes | 0 ok/help/version, 1 parse/arg/run errors | ✓ (clap's default 2 is remapped to 1) |
| Status lines | `Encoding [VarDCT\|Modular\|JPEG, dX.ddd\|lossless\|lossless transcode, effort: N]`, `Compressed to N bytes (x.xxx bpp).`, `Using N threads, average speed: X MP/s.` | ✓ reproduced |

## jixel-only extensions (no cjxl equivalent — not compared)

`--lossless` (alias of `-d 0`), `--fast-lossless`, `--splines`,
`--color-space <srgb|srgb-linear|display-p3|bt2020-pq|bt2020-hlg>`,
`--icc-profile <FILE>`, `--orientation <1-8>`.

Note: cjxl expresses color hints as `-x color_space=...` / `-x icc_pathname=...`;
jixel-cli exposes dedicated flags instead. The cjxl `-x` shorthand names
(`sRGB`, `DisplayP3`, `Rec2100PQ`, `Rec2100HLG`) correspond to `srgb`,
`display-p3`, `bt2020-pq`, `bt2020-hlg`; **`Adobe98`/`ProPhoto` have no jixel
preset** and are rejected.

## cjxl flags deliberately absent (no jixel API)

`-a/--alpha_distance`, `--group_order`, `--center_x/--center_y`,
`--compress_boxes`, `--brotli_effort`, `--dec-hints (-x)`,
`--photon_noise_iso`, `--min_nits`/tone-mapping fields,
`--resampling/--ec_resampling`, `--epf`, `--gaborish`, `--noise`, `--dots`,
`--keep_invisible`, `--premultiply`, `--override_bitdepth`,
`--upsampling_mode`, `--already_downsampled`, `--progressive_ac`,
`--qprogressive_ac`, `--progressive_dc`, `--responsive (-R)`,
all modular tuning (`-I`, `-C`, `-g`, `-P`, `-E`, `-X`, `-Y`,
`--modular_palette_colors`, `--modular_lossy_palette`),
`--codestream_level`, `--buffering`, `--output_mode`, `--streaming_input/output`,
`--disable_output`, `--num_reps`, `--frame_indexing`,
`--jpeg_reconstruction_cfl`, `--allow_expert_options`,
`--disable_perceptual_optimizations`.

Passing an unknown flag exits 1 with clap's message (never silently ignored).

## Input-format note

cjxl reads PNG/JPEG/GIF/PNM (+APNG) and JXL; jixel-cli reads whatever the
`image` crate decodes (PNG, JPEG, GIF, BMP, TIFF, WebP, EXR, HDR, …) but not
PPM/PGM/PNM — raw PNM has no decoder here, and `-x` color hints are the
main use case for it in cjxl.

## Known behavioral differences (jixel library limits)

* Float inputs (EXR/HDR): lossy only; `-d 0`/`--lossless` errors instead of
  silently requantizing (libjxl supports lossless float via modular).
* Effort granularity: 3 tiers, not 10.
* `--faster_decoding`: 3 levels, not 5.
* Container cannot be forced (`--container=1` errors).
