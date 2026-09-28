# jixel-cli test matrix (cjxl-aligned interface). Validates output signatures
# and expected success/failure per case.
$ErrorActionPreference = "Continue"
Set-Location "C:\Users\lamff\Documents\Code\test\jixel-cli"
$exe = ".\target\release\jixel-cli.exe"
$dir = "test-output"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
Remove-Item "$dir\*.jxl" -ErrorAction SilentlyContinue

$pass = 0; $fail = 0

function Run-Case {
    param($Name, [string[]]$Params, [switch]$Fail, [string]$Sig = "any")
    $prevPass = $script:pass; $prevFail = $script:fail
    Write-Host "--- $Name ---" -ForegroundColor Cyan
    & $exe @Params --quiet
    $code = $LASTEXITCODE
    if ($Fail) {
        if ($code -ne 0) { Write-Host "OK  rejected as expected (exit=$code)" -ForegroundColor Green; $script:pass++ }
        else { Write-Host "FAIL accepted invalid input" -ForegroundColor Red; $script:fail++ }
        return
    }
    if ($code -ne 0) { Write-Host "FAIL exit=$code" -ForegroundColor Red; $script:fail++; return }
    $out = $Params[1]
    if (-not (Test-Path $out)) { Write-Host "FAIL no output file" -ForegroundColor Red; $script:fail++; return }
    $b = [System.IO.File]::ReadAllBytes((Resolve-Path $out).Path)
    $sig = if ($b[0] -eq 0xFF -and $b[1] -eq 0x0A) { "bare" }
        elseif ($b.Length -ge 12 -and $b[0] -eq 0 -and $b[4] -eq 0x4A -and $b[5] -eq 0x58 -and $b[6] -eq 0x4C -and $b[7] -eq 0x20) { "container" }
        else { "BAD" }
    if ($Sig -ne "any" -and $sig -ne $Sig) {
        Write-Host "FAIL expected signature $Sig, got $sig" -ForegroundColor Red; $script:fail++; return
    }
    if ($sig -eq "BAD") { Write-Host "FAIL bad magic" -ForegroundColor Red; $script:fail++; return }
    Write-Host ("OK  {0} bytes ({1})" -f $b.Length, $sig) -ForegroundColor Green
    $script:pass++
}

$img = "test-images\rgb.png"
$rgba = "test-images\rgba.png"
$opq = "test-images\rgba_opaque.png"

# --- lossy modes ---
Run-Case "lossy-default"     @($img, "$dir\lossy-default.jxl")
Run-Case "lossy-d05"         @($img, "$dir\lossy-d05.jxl", "-d", "0.5")
Run-Case "lossy-q85"         @($img, "$dir\lossy-q85.jxl", "-q", "85")
Run-Case "lossy-e2"          @($img, "$dir\lossy-e2.jxl", "-e", "2")
Run-Case "slow-e9"           @($img, "$dir\slow-e9.jxl", "-e", "9")
Run-Case "splines-e9"        @($img, "$dir\splines-e9.jxl", "-e", "9", "--splines")
Run-Case "modular-lossy"     @($img, "$dir\modular-lossy.jxl", "-e", "9", "-m", "1")
Run-Case "modular-auto"      @($img, "$dir\modular-auto.jxl", "-e", "9", "-m", "2")
Run-Case "progressive"       @($img, "$dir\progressive.jxl", "-p")
Run-Case "no-patches"        @($img, "$dir\no-patches.jxl", "--patches", "0")
Run-Case "orient-6"          @($img, "$dir\orient-6.jxl", "--orientation", "6")
Run-Case "p3-color"          @($img, "$dir\p3-color.jxl", "--color-space", "display-p3")

# --- lossless modes ---
Run-Case "lossless-q100"     @($img, "$dir\lossless-q100.jxl", "-q", "100")
Run-Case "lossless-d0"       @($img, "$dir\lossless-d0.jxl", "-d", "0")
Run-Case "fast-lossless"     @($img, "$dir\fast-lossless.jxl", "--fast-lossless")
Run-Case "threads-0"         @($img, "$dir\threads-0.jxl", "-q", "100", "--num_threads", "0")
Run-Case "threads-4"         @($img, "$dir\threads-4.jxl", "-q", "100", "--num_threads", "4")
Run-Case "fd3-lossless"      @($img, "$dir\fd3-lossless.jxl", "-q", "100", "--faster_decoding", "3")

# --- alpha / formats ---
Run-Case "rgba-default"      @($rgba, "$dir\rgba-default.jxl")
Run-Case "rgba-lossless"     @($rgba, "$dir\rgba-lossless.jxl", "-q", "100")
Run-Case "rgba-strip1"       @($opq, "$dir\rgba-strip1.jxl", "-q", "100", "--strip_alpha", "1")
Run-Case "rgbaop-lossy"      @($opq, "$dir\rgbaop-lossy.jxl")
Run-Case "gray-lossless"     @("test-images\gray.png", "$dir\gray-lossless.jxl", "-q", "100")
Run-Case "gray-lossy"        @("test-images\gray.png", "$dir\gray-lossy.jxl")
Run-Case "gray16-lossless"   @("test-images\gray16.png", "$dir\gray16-lossless.jxl", "-q", "100")
Run-Case "gray16-lossy"      @("test-images\gray16.png", "$dir\gray16-lossy.jxl")
Run-Case "gray16-fl"         @("test-images\gray16.png", "$dir\gray16-fl.jxl", "--fast-lossless")
Run-Case "graya16-lossless"  @("test-images\graya16.png", "$dir\graya16-lossless.jxl", "-q", "100")
Run-Case "rgb16-lossless"    @("test-images\rgb16.png", "$dir\rgb16-lossless.jxl", "-q", "100")
Run-Case "rgb16-fl"          @("test-images\rgb16.png", "$dir\rgb16-fl.jxl", "--fast-lossless")
Run-Case "rgb16-lossy"       @("test-images\rgb16.png", "$dir\rgb16-lossy.jxl")
Run-Case "rgb16-e2"          @("test-images\rgb16.png", "$dir\rgb16-e2.jxl", "-e", "2")
Run-Case "rgba16-lossless"   @("test-images\rgba16.png", "$dir\rgba16-lossless.jxl", "-q", "100")
Run-Case "rgba16-lossy"      @("test-images\rgba16.png", "$dir\rgba16-lossy.jxl")
Run-Case "rgbf-exr"          @("test-images\rgbf.exr", "$dir\rgbf-exr.jxl")

# --- JPEG behavior (cjxl semantics) ---
Run-Case "jpeg-implicit"     @("test-images\rgb.jpg", "$dir\jpeg-implicit.jxl") -Sig "container"
Run-Case "jpeg-pixels"       @("test-images\rgb.jpg", "$dir\jpeg-pixels.jxl", "-j", "0")
Run-Case "jpeg-pixels-lossless" @("test-images\rgb.jpg", "$dir\jpeg-pixels-lossless.jxl", "-j", "0", "-q", "100")
Run-Case "jpeg-no-jbrd"      @("test-images\rgb.jpg", "$dir\jpeg-no-jbrd.jxl", "--allow_jpeg_reconstruction", "0") -Sig "bare"

# --- error cases (must exit non-zero) ---
Run-Case "err-jpeg-dist"     @("test-images\rgb.jpg", "$dir\neg1.jxl", "-d", "1.0") -Fail
Run-Case "err-q-and-d"       @($img, "$dir\neg2.jxl", "-d", "1", "-q", "90") -Fail
Run-Case "err-container1"    @($img, "$dir\neg3.jxl", "--container", "1") -Fail
Run-Case "err-effort11"      @($img, "$dir\neg4.jxl", "-e", "11") -Fail
Run-Case "err-fl-on-jpeg"    @("test-images\rgb.jpg", "$dir\neg5.jxl", "--fast-lossless") -Fail
Run-Case "err-dist-range"    @($img, "$dir\neg6.jxl", "-d", "30") -Fail

Write-Host ""
Write-Host "Summary: $pass passed, $fail failed" -ForegroundColor $(if ($fail) { "Red" } else { "Green" })
if ($fail) { exit 1 }

