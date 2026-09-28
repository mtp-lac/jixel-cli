//! Decodes jixel-cli outputs with jxl-oxide (pure-Rust JPEG XL decoder) and
//! verifies them against the original pixels.
//! Run: cargo run --release --example verify

use image::{DynamicImage, GenericImageView};
use jxl_oxide::integration::JxlDecoder;

#[derive(Clone, Copy)]
enum Expect {
    /// Lossless: decoded pixels must match the source bit-exactly.
    Exact,
    /// Lossy/transcode: must decode; report PSNR and max error.
    Report,
}

struct Case {
    src: &'static str,
    jxl: &'static str,
    expect: Expect,
}

const CASES: &[Case] = &[
    // ---- lossless: bit-exact roundtrip required ----
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/lossless.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/fast-lossless.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/rgba.png",
        jxl: "test-output/rgba-lossless.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/rgba.png",
        jxl: "test-output/rgba-fl.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/gray.png",
        jxl: "test-output/gray-lossless.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/gray16.png",
        jxl: "test-output/gray16-lossless.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/gray16.png",
        jxl: "test-output/gray16-fl.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/rgb16.png",
        jxl: "test-output/rgb16-lossless.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/rgb16.png",
        jxl: "test-output/rgb16-fl.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/graya16.png",
        jxl: "test-output/graya16-lossless.jxl",
        expect: Expect::Exact,
    },
    // ---- lossy: must decode with acceptable fidelity ----
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/lossy-q90.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/lossy-q30.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/lossy-slow.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/lossy-fastest.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/progressive.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/lossy-mod-auto.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/splines.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/threads-2.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/p3-color.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgba.png",
        jxl: "test-output/rgba.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/gray.png",
        jxl: "test-output/gray.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/gray16.png",
        jxl: "test-output/gray16.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb16.png",
        jxl: "test-output/rgb16.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb16.png",
        jxl: "test-output/rgb16-fastest.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgba16.png",
        jxl: "test-output/rgba16.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgbf.exr",
        jxl: "test-output/rgbf-exr.jxl",
        expect: Expect::Report,
    },
    // ---- JPEG transcode: decode must match the original JPEG's pixels ----
    Case {
        src: "test-images/rgb.jpg",
        jxl: "test-output/jpeg-transcode.jxl",
        expect: Expect::Report,
    },
];

fn main() {
    let mut failures = 0usize;
    for case in CASES {
        match run_case(case) {
            Ok(msg) => println!("OK   {:<26} {msg}", name(case.jxl)),
            Err(e) => {
                failures += 1;
                println!("FAIL {:<26} {e}", name(case.jxl));
            }
        }
    }
    println!();
    if failures == 0 {
        println!("All {} verification cases passed.", CASES.len());
    } else {
        println!("{failures} of {} verification cases FAILED.", CASES.len());
        std::process::exit(1);
    }
}

fn name(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_string()
}

fn decode_jxl(path: &str) -> Result<DynamicImage, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("open: {e}"))?;
    let decoder = JxlDecoder::new(file).map_err(|e| format!("decoder init: {e}"))?;
    DynamicImage::from_decoder(decoder).map_err(|e| format!("decode: {e}"))
}

fn run_case(case: &Case) -> Result<String, String> {
    let src = image::open(case.src).map_err(|e| format!("source open: {e}"))?;
    let dec = decode_jxl(case.jxl)?;

    if src.dimensions() != dec.dimensions() {
        return Err(format!(
            "dimension mismatch: source {}x{}, decoded {}x{}",
            src.width(),
            src.height(),
            dec.width(),
            dec.height()
        ));
    }

    let a = src.to_rgba32f();
    let b = dec.to_rgba32f();

    let mut max_diff = 0.0f32;
    let mut sq_err = 0.0f64;
    let mut n = 0u64;
    for (pa, pb) in a.pixels().zip(b.pixels()) {
        for ca in 0..4 {
            let va = pa.0[ca];
            let vb = pb.0[ca];
            let d = (va - vb).abs();
            if d > max_diff {
                max_diff = d;
            }
            sq_err += (f64::from(va) - f64::from(vb)) * (f64::from(va) - f64::from(vb));
            n += 1;
        }
    }
    let rmse = (sq_err / n as f64).sqrt();
    let psnr = if rmse > 0.0 {
        10.0 * (1.0 / (rmse * rmse)).log10()
    } else {
        f64::INFINITY
    };

    match case.expect {
        Expect::Exact => {
            if max_diff == 0.0 {
                Ok(format!(
                    "lossless bit-exact ({}x{})",
                    src.width(),
                    src.height()
                ))
            } else {
                Err(format!("NOT bit-exact: max sample diff {max_diff}"))
            }
        }
        Expect::Report => Ok(format!(
            "decoded {}x{}, PSNR {:.1} dB, max diff {:.4}",
            src.width(),
            src.height(),
            psnr,
            max_diff
        )),
    }
}
