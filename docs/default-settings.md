# Default settings

The CLI exposes cjxl-compatible defaults; where jixel's own `EncodeConfig` default
differs, the CLI follows cjxl (noted below). The underlying jixel library defaults
are documented in its `EncodeConfig::default()` and related `Default` impls.

## Effective CLI defaults

| Option | Default |
|---|---|
| `--distance` | 1.0 (JPEG/GIF input: 0.0 = lossless transcode) |
| `--effort` | 7 → jixel `Speed::Fast` |
| `--modular` | VarDCT (`LossyModular::Off`) |
| `--progressive` | off |
| `--progressive_dc` | off |
| `--progressive_dc_levels` | unset (`None`, squeeze to 8x8 preview) |
| `--group_order` | `scanline` |
| `--patches` | on, auto-disabled with `--progressive` (cjxl rule) |
| `--dots` | off |
| `--splines` | off |
| `--learned-rate` | on (jixel default) |
| `--arbiter` | `ssimulacra2` (jixel default) |
| `--coding-transform` | `xyb` (jixel default) |
| `--faster_decoding` | 0 → `DecodingSpeed::Slow` |
| `--strip_alpha` | -1 (encoder chooses) |
| `--num_threads` | -1 (machine default) |
| `--intensity_target` | 0 (auto) |
| `--color-space` | `srgb` |

## jixel library defaults (`EncodeConfig::default()`)

| Field | Default |
|---|---|
| `distance` | 1.0 |
| `speed` | `Speed::Fast` |
| `decoding_speed` | `DecodingSpeed::Slow` |
| `lossy_modular` | `LossyModular::Off` |
| `arbiter` | `Arbiter::Ssimulacra2` |
| `coding_transform` | `CodingTransform::Xyb` |
| `progressive` / `progressive_dc` | false / false |
| `progressive_dc_levels` | `None` (squeeze to 8x8 preview) |
| `group_order` | `GroupOrder::Scanline` |
| `patches` | false *(CLI turns on by default per cjxl)* |
| `splines` / `dots` | false / false |
| `learned_rate` | true |
| `intensity_target` / `min_nits` | `None` / 0.0 |
| `relative_to_max_display` / `linear_below` | false / 0.0 |
| `fill_invisible` | true |
| `orientation` | `Orientation::Normal` |
| `num_threads` | machine parallel capacity |
| `gain_map` | `None` |

`JpegTranscodeConfig::default()` (the lossless-JPEG path): `jpeg_reconstruction=true`,
`num_threads=1`, `speed=Speed::Fast`. `FlMeta::default()` (the fast-lossless path):
sRGB (D65 / BT.709 / sRGB transfer / Relative intent), `alpha_premultiplied=false`,
`orientation=Normal`.
