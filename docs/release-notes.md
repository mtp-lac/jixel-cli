# Release notes

## jixel 0.5.0

This project tracks upstream `awxkee/jixel` and ships it unmodified
(jixel-cli upgraded `0.4.2` -> `0.5.0`).

**No new public API in 0.5.0, so no new flags.** The `EncodeConfig` field set,
the `Speed`/`Arbiter`/`GroupOrder`/`CodingTransform` enums and every exported
encoder function are unchanged from 0.4.2; this is a drop-in dependency bump.

**Encoder changes that come along with the bump (no flag needed):**

- **New transform selection.** Upstream added DCT 32x8 / 8x32 transforms to the
  lossy VarDCT arm.
- **Improved ink layers and region frames.** The special-case layer coders got
  reworked; very-low-bit-depth / sparse content can encode differently.
- **Fixes and tuning.** A dequant-bias correction plus assorted speed/density
  tweaks across the modular and VarDCT paths.
- **Feature restructuring.** `cvvdp-lite` and `ssimu2-lite` (the
  SSIMULACRA 2 / ColorVideoVDP metric crates) moved behind a new `extraslow`
  Cargo feature, which remains part of jixel's default features — so
  `--arbiter ssimulacra2|cvvdp` works exactly as before.

Because the encoder internals changed, encoded output (byte sizes / streams) can
differ from jixel 0.4.2; this is expected.

## jixel 0.4.2

This project tracks upstream `awxkee/jixel` and ships it unmodified
(jixel-cli upgraded `0.4.1` -> `0.4.2`).

**New public API in 0.4.2, now exposed as flags:**

- **`--progressive_dc <0|1>`** - upstream added `EncodeConfig::progressive_dc`
  (`with_progressive_dc`): code the lossy VarDCT frame's 1:8 DC plane as its own
  squeezed modular frame ahead of the main frame, so a decoder can show previews
  refining from 1:256 to 1:8 before any AC data arrives. Costs some size; applies
  to XYB frames without alpha only. This is the jixel equivalent of cjxl's
  `--progressive_dc`.
- **`--progressive_dc_levels <LEVELS>`** - `EncodeConfig::progressive_dc_levels`
  (`with_progressive_dc_levels`): limit how many squeeze levels the progressive DC
  frame keeps (finest first; each level halves the preview resolution). jixel-only
  (cjxl has no level control). Requires `--progressive_dc=1`.
- **`--group_order <scanline|center-first|diagonals>`** - upstream added the
  `GroupOrder` enum and `EncodeConfig::group_order` (`with_group_order`): the order
  in which a lossy VarDCT frame stores its groups for progressive rendering.
  `scanline` (default) and `center-first` match cjxl's `--group_order=0/1`;
  `diagonals` is a jixel extension.

All three are **lossy-only** and are rejected (not silently ignored) at lossless
distance, with `--fast-lossless`, and on the lossless-JPEG-transcode path.

**Encoder changes that come along with the bump (no flag needed):**

- **SSIMULACRA2 moved to the external `ssimu2-lite` crate.** jixel 0.4.1 built
  SSIMULACRA2 in-tree; 0.4.2 uses the same `ssimu2-lite` dependency that already
  supplies CVVDP (as `cvvdp-lite`), cutting in-tree code and unifying the metric
  paths. The arbiter default and per-arbiter tuning rows are unchanged, so
  default output remains equivalent to 0.4.1.
- **Internal reorganisation / speedups.** The adaptive-quantisation modules were
  consolidated (`aq_blur`/`aq_loop` folded into `adaptive_quant`) and several hot
  encode paths pass sample buffers by value instead of by reference.

## jixel 0.4.1

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
- **All six speed tiers are now reachable via `-e/--effort`** - jixel 0.4.0 added
  `UltraFast`, `Medium` and `ExtraSlow` to the original three, but the CLI still
  collapsed `--effort` onto 3 of them. The mapping now covers `UltraFast`
  (effort 1), `Fastest` (2-3), `Fast` (4-7), `Medium` (8), `Slow` (9) and
  `ExtraSlow` (10). See [docs/effort.md](<effort.md>).

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
`progressive_shifts` (cjxl's `--progressive_ac` and `--qprogressive_ac`;
`--progressive_dc` is exposed via `--progressive_dc`), `brotli_compression`
(Brotli-compressed `brob` boxes), `fill_invisible`, CMYK input
(`encode_image_cmyk*` / `InkConvention`), and the remaining tone-mapping fields
`min_nits`, `relative_to_max_display`, `linear_below`. See
[docs/cjxl-alignment.md](<cjxl-alignment.md>) for the full cjxl comparison.
