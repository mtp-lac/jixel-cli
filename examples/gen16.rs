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

    println!("wrote 16-bit + float test images to {out_dir}/");
}
