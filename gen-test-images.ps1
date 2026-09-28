# Generate small test images for jixel-cli
Add-Type -AssemblyName System.Drawing
$out = "C:\Users\lamff\Documents\Code\test\jixel-cli\test-images"
New-Item -ItemType Directory -Force -Path $out | Out-Null

function Fill-Bitmap($bmp, $fn) {
    for ($y = 0; $y -lt $bmp.Height; $y++) {
        for ($x = 0; $x -lt $bmp.Width; $x++) {
            $bmp.SetPixel($x, $y, (& $fn $x $y))
        }
    }
}

$W = 128; $H = 96

# RGB8 gradient
$bmp = New-Object System.Drawing.Bitmap($W, $H, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
Fill-Bitmap $bmp { param($x,$y) [System.Drawing.Color]::FromArgb([int](255*$x/($W-1)), [int](255*$y/($H-1)), 128) }
$bmp.Save("$out\rgb.png", [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Save("$out\rgb.jpg", [System.Drawing.Imaging.ImageFormat]::Jpeg)
$bmp.Dispose()

# RGBA8 with gradient alpha
$bmp = New-Object System.Drawing.Bitmap($W, $H, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
Fill-Bitmap $bmp { param($x,$y) [System.Drawing.Color]::FromArgb([int](255*$y/($H-1)), [int](255*$x/($W-1)), 64, 200) }
$bmp.Save("$out\rgba.png", [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

# Grayscale8
$bmp = New-Object System.Drawing.Bitmap($W, $H, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
Fill-Bitmap $bmp { param($x,$y) $v = [int](255*$x/($W-1)); [System.Drawing.Color]::FromArgb($v, $v, $v) }
$bmp.Save("$out\gray.png", [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

# 16-bit PNGs are generated separately by `cargo run --example gen16` (GDI+ cannot
# write 16 bpc PNGs).

Get-ChildItem $out | ForEach-Object { Write-Host ("{0,-12} {1,8} bytes" -f $_.Name, $_.Length) }