//! Decodes jixel-cli outputs with jxl-oxide (pure-Rust JPEG XL decoder) and
//! verifies them against the original pixels.
//! Run: cargo run --release --example verify

use image::{DynamicImage, GenericImageView};
use jxl_oxide::integration::JxlDecoder;

#[derive(Clone, Copy, PartialEq)]
enum Expect {
    /// Lossless: decoded pixels must match the source bit-exactly.
    Exact,
    /// Lossy/transcode: must decode; report PSNR and max error.
    Report,
    /// Must decode without error only (e.g. orientation swaps dimensions).
    DecodeOnly,
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
        jxl: "test-output/lossless-q100.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/lossless-d0.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/threads-0.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/threads-4.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/fd3-lossless.jxl",
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
        src: "test-images/rgba_opaque.png",
        jxl: "test-output/rgba-strip1.jxl",
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
        src: "test-images/graya16.png",
        jxl: "test-output/graya16-lossless.jxl",
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
        src: "test-images/rgba16.png",
        jxl: "test-output/rgba16-lossless.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/rgb.jpg",
        jxl: "test-output/jpeg-pixels-lossless.jxl",
        expect: Expect::Exact,
    },
    // f32 lossless (jixel encode_f32_lossless_rgba): IEEE-754 bit-exact
    Case {
        src: "test-images/rgbf.exr",
        jxl: "test-output/rgbf-lossless.jxl",
        expect: Expect::Exact,
    },
    Case {
        src: "test-images/rgbaf.exr",
        jxl: "test-output/rgbaf-lossless.jxl",
        expect: Expect::Exact,
    },
    // ---- lossy: must decode with acceptable fidelity ----
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/lossy-default.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/lossy-d05.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/lossy-q85.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/lossy-e2.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/slow-e9.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/splines-e9.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/modular-lossy.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/modular-auto.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/progressive.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/no-patches.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/p3-color.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgba.png",
        jxl: "test-output/rgba-default.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgba_opaque.png",
        jxl: "test-output/rgbaop-lossy.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/gray.png",
        jxl: "test-output/gray-lossy.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/gray16.png",
        jxl: "test-output/gray16-lossy.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb16.png",
        jxl: "test-output/rgb16-lossy.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb16.png",
        jxl: "test-output/rgb16-e2.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgba16.png",
        jxl: "test-output/rgba16-lossy.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgbf.exr",
        jxl: "test-output/rgbf-exr.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgbaf.exr",
        jxl: "test-output/rgbaf-lossy.jxl",
        expect: Expect::Report,
    },
    // ---- JPEG paths ----
    Case {
        src: "test-images/rgb.jpg",
        jxl: "test-output/jpeg-implicit.jxl",
        expect: Expect::Report,
    },
    Case {
        src: "test-images/rgb.jpg",
        jxl: "test-output/jpeg-pixels.jxl",
        expect: Expect::Report,
    },
    // ---- orientation changes rendered size ----
    Case {
        src: "test-images/rgb.png",
        jxl: "test-output/orient-6.jxl",
        expect: Expect::DecodeOnly,
    },
];

fn main() {
    // Every case reads an encoder output produced by the test matrix
    // (run-tests.ps1) plus its source fixture. If none of the outputs exist,
    // the matrix was never run — report that instead of 41 identical
    // "path not found" failures.
    if !CASES.iter().any(|c| std::path::Path::new(c.jxl).exists()) {
        eprintln!(
            "ERROR: no .jxl files found under test-output/ — run the encode \
             matrix (run-tests.ps1) before verify."
        );
        std::process::exit(2);
    }

    let mut failures = 0usize;
    let mut missing_outputs = 0usize;
    let mut missing_sources = 0usize;
    for case in CASES {
        // Report *both* missing files up front: a fixture the matrix did not
        // create (e.g. gen-test-images.ps1 not run) is otherwise indistinguishable
        // from a genuinely unencoded output.
        let src_missing = !std::path::Path::new(case.src).exists();
        let jxl_missing = !std::path::Path::new(case.jxl).exists();
        if src_missing {
            missing_sources += 1;
        }
        if jxl_missing {
            missing_outputs += 1;
        }
        match run_case(case) {
            Ok(msg) => println!("OK   {:<28} {msg}", name(case.jxl)),
            Err(e) => {
                failures += 1;
                println!("FAIL {:<28} {e}", name(case.jxl));
            }
        }
    }
    println!();
    if failures == 0 {
        println!("All {} verification cases passed.", CASES.len());
    } else {
        if missing_sources > 0 {
            eprintln!(
                "note: {missing_sources} case(s) missing source fixtures — run \
                 gen-test-images.ps1 / gen16 before verify."
            );
        }
        if missing_outputs > 0 {
            eprintln!(
                "note: {missing_outputs} case(s) missing test-output/*.jxl — run \
                 run-tests.ps1 before verify."
            );
        }
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

    if case.expect == Expect::DecodeOnly {
        return Ok(format!("decoded {}x{}", dec.width(), dec.height()));
    }

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
            let diff = (va - vb).abs();
            if diff > max_diff {
                max_diff = diff;
            }
            let df = f64::from(va) - f64::from(vb);
            sq_err += df * df;
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
        _ => Ok(format!(
            "decoded {}x{}, PSNR {:.1} dB, max diff {:.4}",
            src.width(),
            src.height(),
            psnr,
            max_diff
        )),
    }
}
