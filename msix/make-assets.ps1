<#
.SYNOPSIS
  PerformanceTray のロゴ画像を生成する。
.DESCRIPTION
  ロゴは GDI+ の図形描画で作る。画像ファイルをリポジトリに置かずに済み、
  どのサイズでも輪郭がくっきり出る。意匠は使用率を表す 2 本の縦棒（左が CPU、右がメモリのつもり）。
.PARAMETER OutDir
  MSIX パッケージ用のロゴを書き出す場所。
.PARAMETER StoreLogoDir
  指定するとストア掲載用の大きいロゴもここに書き出す。
#>
param(
    [string]$OutDir = (Join-Path $PSScriptRoot '..\obj\msix\Assets'),
    [string]$StoreLogoDir
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

# AppxManifest.xml の BackgroundColor と合わせること
$ColorFrom = [System.Drawing.Color]::FromArgb(0x1F, 0xA8, 0x7A)
$ColorTo   = [System.Drawing.Color]::FromArgb(0x0E, 0x5E, 0x46)

function Add-RoundedRect {
    # 角丸四角のパスを作る
    param([double]$X, [double]$Y, [double]$W, [double]$H, [double]$R)
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = [single]($R * 2)
    $path.AddArc([single]$X, [single]$Y, $d, $d, 180, 90)
    $path.AddArc([single]($X + $W - $d), [single]$Y, $d, $d, 270, 90)
    $path.AddArc([single]($X + $W - $d), [single]($Y + $H - $d), $d, $d, 0, 90)
    $path.AddArc([single]$X, [single]($Y + $H - $d), $d, $d, 90, 90)
    $path.CloseFigure()
    return $path
}

function New-Logo {
    <#
      2 本の縦棒のロゴを 1 枚描く。
      $Width と $Height が違う場合（ワイドタイル）は中央に正方形として描く。
      $Padding は図形の周囲に空ける余白の割合。タイルは余白を広めに取る。
    #>
    param([int]$Width, [int]$Height, [double]$Padding = 0.18)

    $bmp = New-Object System.Drawing.Bitmap $Width, $Height
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias

    $full = New-Object System.Drawing.Rectangle 0, 0, $Width, $Height
    $bg = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        $full, $ColorFrom, $ColorTo, [System.Drawing.Drawing2D.LinearGradientMode]::ForwardDiagonal)
    $g.FillRectangle($bg, $full)
    $bg.Dispose()

    # 図形は短辺を基準にした正方形の中に収める
    $s = [Math]::Min($Width, $Height) * (1.0 - $Padding * 2)
    $left = ($Width - $s) / 2.0
    $bottom = ($Height + $s) / 2.0

    $white = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
    $faint = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(0x59, 0xFF, 0xFF, 0xFF))

    # 2 本の棒。薄い全高の枠の中に、使用率ぶんの高さだけ白く塗る
    $barW = $s * 0.30
    $gap = $s * 0.16
    $x1 = $left + ($s - (2 * $barW + $gap)) / 2.0
    $x2 = $x1 + $barW + $gap
    $radius = $barW * 0.22
    foreach ($bar in @(@($x1, 0.58), @($x2, 0.82))) {
        $x, $level = $bar
        $frame = Add-RoundedRect -X $x -Y ($bottom - $s) -W $barW -H $s -R $radius
        $g.FillPath($faint, $frame)
        $frame.Dispose()
        $h = $s * $level
        $fill = Add-RoundedRect -X $x -Y ($bottom - $h) -W $barW -H $h -R $radius
        $g.FillPath($white, $fill)
        $fill.Dispose()
    }

    $faint.Dispose()
    $white.Dispose()
    $g.Dispose()
    return $bmp
}

function Save-Logo {
    param([string]$Path, [int]$Width, [int]$Height, [double]$Padding)
    $bmp = New-Logo -Width $Width -Height $Height -Padding $Padding
    $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$tiles = [ordered]@{
    'Square44x44Logo.png'   = @(44, 44)
    'Square71x71Logo.png'   = @(71, 71)
    'Square150x150Logo.png' = @(150, 150)
    'Square310x310Logo.png' = @(310, 310)
    'Wide310x150Logo.png'   = @(310, 150)
    'StoreLogo.png'         = @(50, 50)
}
foreach ($name in $tiles.Keys) {
    $w, $h = $tiles[$name]
    Save-Logo -Path (Join-Path $OutDir $name) -Width $w -Height $h -Padding 0.18
}
"パッケージ用ロゴ: $OutDir"

if ($StoreLogoDir) {
    New-Item -ItemType Directory -Force -Path $StoreLogoDir | Out-Null
    # ストア掲載用。300x300 が基本、1080x1080 は大きく表示される場所で使われる
    foreach ($size in 300, 1080) {
        Save-Logo -Path (Join-Path $StoreLogoDir "store-logo-${size}.png") -Width $size -Height $size -Padding 0.16
    }
    "ストア掲載用ロゴ: $StoreLogoDir"
}
