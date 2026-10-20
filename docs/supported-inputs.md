# Supported inputs

PNG, JPEG, GIF, BMP, TIFF, WebP, EXR, HDR… (anything the `image` crate
decodes; no PNM). Bit depths: 8/16-bit integer (L, LA, RGB, RGBA) and
32-bit float (RGB/RGBA), lossy **and** lossless.

Float lossless uses jixel's `encode_f32_lossless_rgba`, which is bit-exact but
v1-limited to finite, non-negative samples; NaN, infinity or negative values
are rejected by the library. Such files need codestream level 10 and are
emitted in container form.
