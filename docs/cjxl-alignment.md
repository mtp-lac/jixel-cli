# cjxl interface alignment

Review of `jixel-cli` against **libjxl's `cjxl`** (main branch,
`tools/cjxl_main.cc`), 2026-09. Goal: same invocation syntax and flag
semantics for everything the jixel 0.3.4 public API can express. jixel-only
extensions are listed separately; flags jixel cannot implement are *not*
silently accepted.

```text
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
| `--strip_alpha <-1\|0\|1\|2>` | -1 encoder chooses (lossy: strip if empty; lossless: keep), 0 keep, 1 strip, 2 strip-if-opaque | ✓ incl. the empty-alpha scan |
| `-p, --progressive` | flag | ✓; like cjxl it also disables `patches` when unspecified |
| `-m, --modular <0\|1>` | 0 VarDCT, 1 Modular (lossy) | ✓; extension value `2`/`auto` = jixel's both-arms heuristic |
| `-j, --lossless_jpeg <0\|1>` (default 1 for JPEG magic) | implicit JPEG transcode; forced off for non-JPEG; error if combined with non-zero distance | ✓ identical auto-detect (FFD8 magic), identical note "Implicit-default for JPEG is lossless-transcoding...", identical conflict error |
| `--allow_jpeg_reconstruction <0\|1>` | store/drop JBRD reconstruction box | ✓ (dropping yields a bare codestream, like cjxl) |
| `--num_threads <-1\|0\|N>` | -1 machine default, 0 no MT | ✓ (0 maps to jixel 1 thread) |
| `--faster_decoding <0..4>` | decode-speed vs density | ✓ mapped to jixel's 3 levels: 0→Slow, 1–2→Fast, 3–4→Fastest (lossless only, same as jixel) |
| `--patches <0\|1>` | encoder chooses default | ✓ default = enabled, auto-disabled with `--progressive` (cjxl rule) |
| `--dots <0\|1>` | encoder chooses default (default: off) | ✓ maps to jixel 0.3.4 `EncodeConfig::dots`; `--dots=1` turns it on. Needs the `splines` Cargo feature (enabled here) and takes effect at effort 9-10 |
| `--intensity_target <nits>` | 0 = auto | ✓ |
| `--container <0\|1>` | 1 forces container | accepts `0`/unset only; `1` **errors**: jixel has no force-container API (metadata auto-switches to container exactly like cjxl's forced promotion) |
| Exit codes | 0 ok/help/version, 1 parse/arg/run errors | ✓ (clap's default 2 is remapped to 1) |
| Status lines | `Encoding [VarDCT\|Modular\|JPEG, dX.ddd\|lossless\|lossless transcode, effort: N]`, `Compressed to N bytes (x.xxx bpp).`, `Using N threads, average speed: X MP/s.` | ✓ reproduced (`--fast-lossless` prints `Encoding [Modular, lossless, fast-lossless]` and omits the thread count, since that encoder has neither knob) |

## jixel-only extensions (no cjxl equivalent — not compared)

`--lossless` (alias of `-d 0`), `--fast-lossless`, `--splines`,
`--learned-rate <0\|1>`, `--color-space <srgb\|srgb-linear\|display-p3\|bt2020-pq\|bt2020-hlg>`,
`--icc-profile <FILE>`, `--orientation <1-8>`.

`--learned-rate` is the switch for jixel's learned rate pricing (jixel 0.3.4
`EncodeConfig::learned_rate`, default on, `Speed::Slow` only). cjxl has no such
flag, so it is a jixel-only extension.

Note: cjxl expresses color hints as `-x color_space=...` / `-x icc_pathname=...`;
jixel-cli exposes dedicated flags instead. The cjxl `-x` shorthand names
(`sRGB`, `DisplayP3`, `Rec2100PQ`, `Rec2100HLG`) correspond to `srgb`,
`display-p3`, `bt2020-pq`, `bt2020-hlg`; **`Adobe98`/`ProPhoto` have no jixel
preset** and are rejected.

## cjxl flags with a jixel API this CLI does not yet expose

These have an upstream counterpart but no jixel-cli switch yet. They are not
accepted, and an unknown flag is not silently ignored either - it exits 1.

| cjxl flag | jixel API | Notes |
|-----------|-----------|-------|
| `--progressive_ac`, `--qprogressive_ac`, `--progressive_dc` | `progressive_passes: Option<u32>`, `progressive_shifts: Option<Vec<u32>>` | jixel takes a pass count or an explicit per-pass coefficient-shift schedule; cjxl's three AC/DC shift knobs are a coarser projection onto it |
| `--compress_boxes`, `--brotli_effort` | `brotli_compression: Option<Arc<dyn BrotliCompression>>` | jixel's hook is for Brotli-compressing EXIF/XMP (`brob`) boxes with a caller-provided compressor; cjxl's effort number and general box policy have no direct equivalent |
| `--min_nits`, `--relative_to_max_display`, `--linear_below` | `min_nits`, `relative_to_max_display`, `linear_below` | Tone-mapping fields exist; only `--intensity_target` is exposed |
| gain-map embedding | `gain_map: Option<GainMap>`, `GainMapFloats`, `IsoGainMap` | Encodes a second codestream plus ISO 21496-1 metadata in a `jhgm` box; forces the container form. cjxl has no such flag either |
| EXIF / XMP embedding | `exif: Option<Vec<u8>>`, `xmp: Option<Vec<u8>>` | Raw TIFF / XMP bytes in `Exif` / `xml ` / `brob` boxes; forces the container form |

## cjxl flags deliberately absent (no jixel API)

`-a/--alpha_distance`, `--group_order`, `--center_x/--center_y`,
`--dec-hints (-x)`, `--photon_noise_iso`,
`--resampling/--ec_resampling`, `--epf`, `--gaborish`, `--noise`,
`--keep_invisible`, `--premultiply`, `--override_bitdepth`,
`--upsampling_mode`, `--already_downsampled`, `--responsive (-R)`,
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

* Float inputs (EXR/HDR): lossy **and** lossless are supported. `-d 0` /
  `-q 100` / `--lossless` route to jixel's `encode_f32_lossless_rgba`, which
  stores each sample's IEEE-754 bits as a modular channel (RGB and RGBA,
  verified bit-exact). That path is v1-limited to **finite, non-negative**
  samples; NaN, infinity or negative values are rejected by the library with
  `f32 lossless v1 supports only finite non-negative values`. f16 input is
  lossy-only (jixel routes lossless only for 32-bit float).
* 32-bit float output requires codestream level 10, so float lossless files
  are emitted in container form.
* Effort granularity: 3 tiers, not 10.
* `--faster_decoding`: 3 levels, not 5.
* Container cannot be forced (`--container=1` errors).

## `--fast-lossless` (jixel extension) accepts only metadata

jixel's `encode_fast_lossless` / `encode_fast_lossless_u16` take just pixels,
dimensions, a color space, an alpha flag and an `FlMeta`. They have no
distance, effort, progressive, patches, decoding-speed, tone-mapping, splines
or thread parameters, so jixel-cli **rejects** those combinations rather than
accepting a flag it would silently drop:

| Combination | Result |
|---|---|
| `--fast-lossless` + `-d` / `-q` / `--lossless` | error (redundant or contradictory) |
| `--fast-lossless` + `-e`, `-m`, `-p`, `--patches`, `--faster_decoding`, `--intensity_target`, `--num_threads`, `--splines` | error: no such control on this encoder |
| `--fast-lossless` + `--strip_alpha`, `--color-space`, `--icc-profile`, `--orientation` | supported (these map onto `FlMeta`) |

Because that encoder is single-threaded, its summary line prints
`Average speed: X MP/s.` without a thread count.
