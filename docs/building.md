# Requirements

- **Rust 1.94+**, `x86_64-pc-windows-msvc` toolchain
- **Visual Studio Build Tools** (C++ workload) for the MSVC linker
- Network on first build (crates.io)

## Building & testing

```powershell
cd jixel-cli
cargo build --release        # -> target\release\jixel-cli.exe
# or full pipeline:
.\build.ps1 -Test
```

Test rig:

```powershell
.\gen-test-images.ps1                        # 8-bit PNG/JPEG fixtures
cargo run --release --example gen16          # 16-bit/float fixtures
.\run-tests.ps1                              # 86-case cjxl-syntax matrix
cargo run --release --example verify         # 53 decode checks (jxl-oxide)
```

`verify` decodes every output with **jxl-oxide** (independent pure-Rust
decoder, dev-dependency): 21 lossless streams must round-trip **bit-exact**
(including 32-bit float RGB and RGBA, and every effort tier), lossy streams must
decode with sane PSNR. Last run: **86/86 encode cases, 53/53 verification
cases**.
