# jixel-cli test matrix: runs every encoding mode and validates output signatures.
$ErrorActionPreference = "Continue"
Set-Location "C:\Users\lamff\Documents\Code\test\jixel-cli"
$exe = ".\target\release\jixel-cli.exe"
$dir = "test-output"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
Remove-Item "$dir\*.jxl" -ErrorAction SilentlyContinue

function Run-Case {
    param($Name, $Params)
    $out = Join-Path $dir "$Name.jxl"
    Write-Host "--- $Name ---" -ForegroundColor Cyan
    & $exe @Params -o $out --quiet
    $code = $LASTEXITCODE
    if ($code -ne 0) { Write-Host "FAIL exit=$code" -ForegroundColor Red; return }
    if (-not (Test-Path $out)) { Write-Host "FAIL no output file" -ForegroundColor Red; return }
    $abs = (Resolve-Path $out).Path
    $b = [System.IO.File]::ReadAllBytes($abs)
    $sig = ""
    if ($b.Length -ge 2 -and $b[0] -eq 0xFF -and $b[1] -eq 0x0A) { $sig = "bare codestream" }
    elseif ($b.Length -ge 12 -and $b[0] -eq 0 -and $b[4] -eq 0x4A -and $b[5] -eq 0x58 -and $b[6] -eq 0x4C -and $b[7] -eq 0x20) { $sig = "container" }
    else { Write-Host ("FAIL bad magic: {0:X2} {1:X2}" -f $b[0], $b[1]) -ForegroundColor Red; return }
    Write-Host ("OK  {0} bytes ({1})" -f $b.Length, $sig) -ForegroundColor Green
}

$img = "test-images\rgb.png"
Run-Case "lossy-q90"     @($img, "-q", "90")
Run-Case "lossy-q30"     @($img, "-q", "30")
Run-Case "lossy-slow"    @($img, "-s", "slow")
Run-Case "lossy-fastest" @($img, "-s", "fastest")
Run-Case "lossless"      @($img, "--lossless")
Run-Case "fast-lossless" @($img, "--fast-lossless")
Run-Case "progressive"   @($img, "--progressive")
Run-Case "lossy-mod-auto" @($img, "-s", "slow", "--lossy-modular", "auto")
Run-Case "splines"       @($img, "-s", "slow", "--splines")
Run-Case "threads-2"     @($img, "-j", "2")
Run-Case "orient-6"      @($img, "--orientation", "6")
Run-Case "p3-color"      @($img, "--color-space", "display-p3")

Run-Case "rgba"          @("test-images\rgba.png")
Run-Case "rgba-lossless"  @("test-images\rgba.png", "--lossless")
Run-Case "rgba-fl"        @("test-images\rgba.png", "--fast-lossless")
Run-Case "rgba-noalpha"  @("test-images\rgba.png", "--no-alpha")
Run-Case "gray"          @("test-images\gray.png")
Run-Case "gray-lossless"  @("test-images\gray.png", "--lossless")
Run-Case "gray16"        @("test-images\gray16.png")
Run-Case "gray16-lossless" @("test-images\gray16.png", "--lossless")
Run-Case "gray16-fl"      @("test-images\gray16.png", "--fast-lossless")
Run-Case "rgb16"         @("test-images\rgb16.png")
Run-Case "rgb16-lossless" @("test-images\rgb16.png", "--lossless")
Run-Case "rgb16-fl"       @("test-images\rgb16.png", "--fast-lossless")
Run-Case "graya16-lossless" @("test-images\graya16.png", "--lossless")
Run-Case "rgb16-fastest"  @("test-images\rgb16.png", "-s", "fastest")
Run-Case "rgba16"        @("test-images\rgba16.png")
Run-Case "rgbf-exr"      @("test-images\rgbf.exr")
Run-Case "jpeg-transcode" @("test-images\rgb.jpg", "--jpeg-lossless")

# Negative tests
Write-Host "--- negative: jpeg-lossless on png ---" -ForegroundColor Cyan
& $exe $img --jpeg-lossless -o "$dir\neg.jxl" 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "OK  rejected (exit=$LASTEXITCODE)" -ForegroundColor Green } else { Write-Host "FAIL accepted bad input" -ForegroundColor Red }

Write-Host "`nAll cases:" -ForegroundColor Yellow
Get-ChildItem $dir | ForEach-Object { Write-Host ("{0,-20} {1,8} bytes" -f $_.Name, $_.Length) }
