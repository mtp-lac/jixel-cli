//! Generates high-bit-depth and float test PNG/EXR files for jixel-cli tests.
//! Run: cargo run --release --example gen16

use image::{ImageBuffer, Luma, LumaA, Rgb, Rgba};

fn main() {
    let out_dir = "test-images";
    std::fs::create_dir_all(out_dir).unwrap();
    let (w, h) = (128u32, 96u32);

    // 16-bit RGB
    let rgb16: ImageBuffer<Rgb<u16>, Vec<u16>> = ImageBuffer::from_fn(w, h, |x, y| {
        Rgb([
            (x * 65535 / (w - 1)) as u16,
            (y * 65535 / (h - 1)) as u16,
            32768,
        ])
    });
    rgb16
        .save(std::path::Path::new(out_dir).join("rgb16.png"))
        .unwrap();

    // 16-bit RGBA
    let rgba16: ImageBuffer<Rgba<u16>, Vec<u16>> = ImageBuffer::from_fn(w, h, |x, y| {
        Rgba([
            (x * 65535 / (w - 1)) as u16,
            (y * 65535 / (h - 1)) as u16,
            60000,
            (x as u16) * 512,
        ])
    });
    rgba16
        .save(std::path::Path::new(out_dir).join("rgba16.png"))
        .unwrap();

    // 16-bit Luma
    let luma16: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_fn(w, h, |x, _y| Luma([(x * 65535 / (w - 1)) as u16]));
    luma16
        .save(std::path::Path::new(out_dir).join("gray16.png"))
        .unwrap();

    // 16-bit LumaA
    let la16: ImageBuffer<LumaA<u16>, Vec<u16>> = ImageBuffer::from_fn(w, h, |x, y| {
        LumaA([(x * 65535 / (w - 1)) as u16, (y * 65535 / (h - 1)) as u16])
    });
    la16.save(std::path::Path::new(out_dir).join("graya16.png"))
        .unwrap();

    // 8-bit RGBA with fully opaque alpha (tests --strip_alpha auto behavior)
    let rgba8: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(w, h, |x, y| {
        Rgba([
            (x * 255 / (w - 1)) as u8,
            (y * 255 / (h - 1)) as u8,
            200,
            255,
        ])
    });
    rgba8
        .save(std::path::Path::new(out_dir).join("rgba_opaque.png"))
        .unwrap();

    // 32-bit float RGB -> OpenEXR
    let rgbf: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::from_fn(w, h, |x, y| {
        Rgb([
            (x as f32) / (w - 1) as f32,
            (y as f32) / (h - 1) as f32,
            0.5f32,
        ])
    });
    rgbf.save_with_format(
        std::path::Path::new(out_dir).join("rgbf.exr"),
        image::ImageFormat::OpenExr,
    )
    .unwrap();

    // 32-bit float RGBA -> OpenEXR (jixel f32 lossless supports alpha)
    let rgbaf: ImageBuffer<Rgba<f32>, Vec<f32>> = ImageBuffer::from_fn(w, h, |x, y| {
        Rgba([
            (x as f32) / (w - 1) as f32,
            (y as f32) / (h - 1) as f32,
            0.25f32,
            (x as f32) / (w - 1) as f32 * 0.5 + 0.5,
        ])
    });
    rgbaf
        .save_with_format(
            std::path::Path::new(out_dir).join("rgbaf.exr"),
            image::ImageFormat::OpenExr,
        )
        .unwrap();

    // Negative float samples: jixel's f32 lossless (v1) only accepts finite
    // non-negative values, so this must be rejected by the library.
    let rgbf_neg: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::from_fn(w, h, |x, y| {
        Rgb([
            (x as f32) / (w - 1) as f32 - 0.5,
            (y as f32) / (h - 1) as f32,
            0.5f32,
        ])
    });
    rgbf_neg
        .save_with_format(
            std::path::Path::new(out_dir).join("rgbf_neg.exr"),
            image::ImageFormat::OpenExr,
        )
        .unwrap();

    println!("wrote 16-bit + float test images to {out_dir}/");
}
