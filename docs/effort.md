# Effort / speed tiers

cjxl's `--effort` is a 1–10 scale; jixel 0.5.x has **six** speed/density tiers,
each a strict superset of the tools of every faster one. `--effort` resolves
onto all six:

| Effort | jixel `Speed` | Character |
|--------|---------------|-----------|
| 1 | `UltraFast` | no transform search at all; plain 8×8 DCT, cheapest entropy coding |
| 2–3 | `Fastest` | 16×16 merges on quiet blocks, prefix-clustered contexts, DC predictor choice |
| 4–7 | `Fast` | square transforms, quantizer refinement, learned DC tree (jixel's default; effort 7 is the CLI default) |
| 8 | `Medium` | full transform and color search of `Slow` without its costliest refinements |
| 9 | `Slow` | every standard coding tool (`--splines`, `--dots`, `--learned-rate`, `--arbiter` engage here) |
| 10 | `ExtraSlow` | all `Slow` tools plus exhaustive final ANS entropy search and wider quantization search |

The mapping is **monotonic** (raising `--effort` never selects a faster, less
thorough tier) and **default-preserving** (effort 7 stays on jixel's own
`#[default]` `Speed::Fast`). It is monotonic in *search effort*, not in byte
count: the encoder optimizes the perceptual metric, so `-e 10` can land a few
bytes above `-e 9` on some images while searching strictly more.

## Effort is meaningful per encode mode

Whether `--effort` changes the output depends on which encoder path is running.

### Lossy pixel encoding (VarDCT; `-d > 0` / `-q < 100`)

All ten levels are meaningful, but only as six distinct outcomes: efforts inside
one tier produce byte-identical files. Verified on `test-images/rgb.png` at
`-d 1` (file size; distinct streams have distinct sizes/MD5s):

| Effort | Tier | Output |
|--------|------|--------|
| 1 | UltraFast | 891 B |
| 2–3 | Fastest | 589 B |
| 4–7 | Fast | 493 B |
| 8 | Medium | 461 B |
| 9 | Slow | 399 B |
| 10 | ExtraSlow | 411 B |

Each slower tier runs strictly more search; results are not monotonic in size —
`-e 10` landed above `-e 9` (411 vs 399 B) here while searching more.

### Lossless pixel encoding (`-d 0` / `--lossless` / `-q 100`)

Only the tiers that enable Modular (lossless) search tools matter:

- **Effort 1–7 (UltraFast / Fastest / Fast): identical output.** None of these
  tiers enable any modular search tool — the per-tier gate table has no modular
  entry before `Medium` — so `-e 1`, `-e 3`, `-e 7` etc. produce byte-identical
  files. Verified on `test-images/rgb.png` (2608 B for every effort 1–7) and
  `test-images/rgb16.png` (2055 B for every effort 1–7).
- **Effort 8 (`Medium`): first modular searches engage** — predictor / RCT /
  weighted-predictor search, learned trees, local/global palettes, and ring-hash
  LZ77 with refined entropy.
- **Effort 9 (`Slow`): adds deeper/denser tree learning** (rank-deep, dense and
  full sampling, dual group layout, context v1) plus the lossy-only additions.
- **Effort 10 (`ExtraSlow`): adds exhaustive LZ77 entropy pricing and priced LZ
  parses** (`lz_exhaustive_entropy`, `lz_priced_search`) plus the lossy closed loops.
- Distinct tier ≠ always distinct bytes: the extra searches can converge. On the
  two test PNGs `-e 8` and `-e 9` produced the same stream (98 B and 160 B
  respectively), while `-e 10` differed again. On harder/synthetic content the
  tiers diverge more often.

### JPEG lossless transcode (default for JPEG input, `--lossless_jpeg=1`)

`--effort` is ignored: this path uses `JpegTranscodeConfig` at its own default
speed and never reads the CLI effort. Verified: `test-images/rgb.jpg` with
`-e 1` and `-e 10` produces byte-identical files.

### `--fast-lossless`

Has no effort control at all; `-e` is rejected rather than silently ignored.

## Features: auto-enabled at high effort vs. respected only via arguments

Raising `--effort` does **not** silently switch on any user-facing feature. The
Slow+ tier (effort 9–10) changes *when already-configured tools run*, not whether
they run. Concretely:

| Feature | Default | Effort gating |
|---|---|---|
| `--patches` | on (unless `--progressive`) | jixel's library auto-enables patches at `Slow`+ inside `with_speed`. jixel-cli always re-sets patches after `with_speed` (cjxl rule: on unless `--progressive`, or the explicit `--patches=0/1`), so through this CLI patch behaviour is uniform across efforts; `--patches=0` is honoured at any effort. |
| `--splines` | off | Requires the flag *and* effort 9–10 *and* the default decoding speed. `--splines` alone at `-e 7` warns and has no effect. |
| `--dots=1` | off | Same gating as splines (effort 9–10, default decoding speed). |
| `--learned-rate` | on | Always accepted; it only changes output at effort 9–10 (it prices transform candidates against the image's own statistics). `--learned-rate=0` disables it at any effort. |
| `--arbiter` | `ssimulacra2` | Always selects the metric, but the choice only changes output at effort 9–10: below that the faster tiers never run the decisions that consult the metric (verified: CVVDP vs SSIMULACRA2 are byte-identical at effort 1–8, different at 9). |
| `-m 2` (`auto`) | n/a | The both-arms heuristic only runs at effort 9–10; below, Auto falls back to VarDCT. `-m 1` (Force) runs at any effort. |
| Lossy-only parametric knobs (`--progressive_dc`, `--group_order`, `--coding-transform`) | — | Not effort-gated: they are honoured at any effort (still lossy-only). |

Features that are implied by effort alone are **internal** search tools, not CLI
switches, e.g. `full_threads` (full thread budget on small images, `Fast`+ —
parallelisation only, never output bytes), `quant_refine`, `rectangles`,
`aq_closed_loop` / `dct128_loop` / `layout_loop` (`ExtraSlow`), and the modular
search set listed above for lossless.
