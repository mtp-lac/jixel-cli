# Effort / speed tiers

cjxl's `--effort` is a 1-10 scale; jixel 0.4.x has **six** speed/density tiers,
each a strict superset of the tools of every faster one. `--effort` resolves
onto all six:

| Effort | jixel `Speed` | Character |
|--------|---------------|-----------|
| 1 | `UltraFast` | no transform search at all; plain 8×8 DCT, cheapest entropy coding |
| 2-3 | `Fastest` | 16×16 merges on quiet blocks, prefix-clustered contexts, DC predictor choice |
| 4-7 | `Fast` | square transforms, quantizer refinement, learned DC tree (jixel's default; effort 7 is the CLI default) |
| 8 | `Medium` | full transform and color search of `Slow` without its costliest refinements |
| 9 | `Slow` | every standard coding tool (`--splines`, `--dots`, `--learned-rate`, `--arbiter` engage here) |
| 10 | `ExtraSlow` | all `Slow` tools plus exhaustive final ANS entropy search and wider quantization search |

The mapping is **monotonic** (raising `--effort` never selects a faster, less
thorough tier) and **default-preserving** (effort 7 stays on jixel's own
`#[default]` `Speed::Fast`). It is monotonic in *search effort*, not in byte
count: the encoder optimizes the perceptual metric, so `-e 10` can land a few
bytes above `-e 9` on some images while searching strictly more. Because jixel
spells its `Slow`-only tools in terms of tier, not effort, `--splines`,
`--dots`, `--learned-rate` and `--arbiter` are documented by their effective
range, **effort 9-10**.
