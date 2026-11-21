# Requirements

- **Rust 1.94+**, `x86_64-pc-windows-msvc` toolchain
- **Visual Studio Build Tools** (C++ workload) for the MSVC linker
- Network on first build (crates.io)

## Versioning & releases

The CLI package version matches the jixel library version. The dependency is
pinned exactly so Cargo cannot silently build a newer library under an older
CLI version. Both `--version` and the encoding banner use this package version.

To update both versions together (Python 3.11+):

```powershell
python .github/scripts/version.py --sync 0.5.9
python .github/scripts/version.py                # checks alignment
```

The release workflow polls upstream stable releases every three hours, syncs
both versions, and publishes `vX.Y.Z` with a
`jixel-cli-jxl-X.Y.Z-windows-x86_64.zip` asset. It also builds the current `main`
once a day as the rolling `nightly` prerelease, with a
`jixel-cli-jxl-X.Y.Z-nightly-windows-x86_64.zip` asset. Nightly refers to the
CLI snapshot; it uses the same pinned crates.io library as the stable build.
Its release notes record the source commit, and its binary reports `X.Y.Z`.
Both channels can be selected manually using the workflow's `channel` input.
Each release verifies the manifest, resolved library, and binary versions.

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
