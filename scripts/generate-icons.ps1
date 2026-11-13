<#
.SYNOPSIS
    生成 AgentControl 应用图标（PNG 全套 + icon.ico）。

.DESCRIPTION
    使用 System.Drawing 绘制一枚「控制中枢」风格图标：深色圆角底 + 蓝色渐变光晕 +
    三个由连线连接的节点，寓意多智能体编排。
    所有尺寸由同一份绘制逻辑按比例缩放，保证各尺寸下都清晰。

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/generate-icons.ps1
#>
param(
    [string]$OutputDir = (Join-Path (Split-Path -Parent $PSScriptRoot) 'src-tauri\icons')
)

Add-Type -AssemblyName System.Drawing

function New-RoundedRectanglePath {
    param([float]$X, [float]$Y, [float]$W, [float]$H, [float]$Radius)
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = $Radius * 2
    $path.AddArc($X, $Y, $d, $d, 180, 90)
    $path.AddArc($X + $W - $d, $Y, $d, $d, 270, 90)
    $path.AddArc($X + $W - $d, $Y + $H - $d, $d, $d, 0, 90)
    $path.AddArc($X, $Y + $H - $d, $d, $d, 90, 90)
    $path.CloseFigure()
    return $path
}

function New-IconBitmap {
    param([int]$Size)

    $bmp = New-Object System.Drawing.Bitmap($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)

    $s = $Size / 512.0
    $radius = 108 * $s
    $pad = 16 * $s

    # 底板：深色圆角矩形
    $bodyPath = New-RoundedRectanglePath -X $pad -Y $pad -W ($Size - 2 * $pad) -H ($Size - 2 * $pad) -Radius $radius
    $bodyBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        (New-Object System.Drawing.Point([int]$pad, [int]$pad)),
        (New-Object System.Drawing.Point([int]($Size - $pad), [int]($Size - $pad))),
        [System.Drawing.Color]::FromArgb(255, 30, 40, 58),
        [System.Drawing.Color]::FromArgb(255, 14, 17, 22)
    )
    $g.FillPath($bodyBrush, $bodyPath)

    # 内部蓝色渐变面板
    $glowPath = New-RoundedRectanglePath -X (24 * $s) -Y (24 * $s) -W ($Size - 48 * $s) -H ($Size - 48 * $s) -Radius (78 * $s)
    $glowBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        (New-Object System.Drawing.Point([int](40 * $s), [int](40 * $s))),
        (New-Object System.Drawing.Point([int]($Size - 40 * $s), [int]($Size - 40 * $s))),
        [System.Drawing.Color]::FromArgb(235, 79, 124, 255),
        [System.Drawing.Color]::FromArgb(215, 58, 102, 232)
    )
    $g.FillPath($glowBrush, $glowPath)

    # 顶部高光（覆盖整块面板，避免出现明显分界）
    $hlBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        (New-Object System.Drawing.Point([int](24 * $s), [int](24 * $s))),
        (New-Object System.Drawing.Point([int](24 * $s), [int]($Size - 24 * $s))),
        [System.Drawing.Color]::FromArgb(90, 255, 255, 255),
        [System.Drawing.Color]::FromArgb(0, 255, 255, 255)
    )
    $g.FillPath($hlBrush, $glowPath)

    # 连线
    $linkPen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(230, 255, 255, 255)), (16 * $s)
    $linkPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $linkPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $g.DrawLine($linkPen, (150 * $s), (150 * $s), (362 * $s), (150 * $s))
    $g.DrawLine($linkPen, (150 * $s), (150 * $s), (256 * $s), (330 * $s))
    $g.DrawLine($linkPen, (362 * $s), (150 * $s), (256 * $s), (330 * $s))

    # 节点
    $nodeBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 255, 255, 255))
    foreach ($p in @(@(150, 150), @(362, 150), @(256, 330))) {
        $r = 44 * $s
        $g.FillEllipse($nodeBrush, ($p[0] * $s - $r), ($p[1] * $s - $r), ($r * 2), ($r * 2))
    }

    # 中心节点强调色
    $accentBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 106, 147, 255))
    $g.FillEllipse($accentBrush, (256 * $s - 30 * $s), (150 * $s - 30 * $s), (60 * $s), (60 * $s))

    $g.Dispose()
    return $bmp
}

function Save-Png {
    param([int]$Size, [string]$Path)
    $bmp = New-IconBitmap -Size $Size
    $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Host ("  {0,-24} {1}x{1}" -f (Split-Path -Leaf $Path), $Size)
}

function Save-Ico {
    param([int[]]$Sizes, [string]$Path)

    $images = @()
    foreach ($size in $Sizes) {
        $bmp = New-IconBitmap -Size $size
        $ms = New-Object System.IO.MemoryStream
        $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        $images += , @{ Size = $size; Data = $ms.ToArray() }
        $ms.Dispose()
        $bmp.Dispose()
    }

    $fs = [System.IO.File]::Open($Path, [System.IO.FileMode]::Create)
    $writer = New-Object System.IO.BinaryWriter($fs)
    $writer.Write([UInt16]0)
    $writer.Write([UInt16]1)
    $writer.Write([UInt16]$images.Count)

    $offset = 6 + 16 * $images.Count
    foreach ($img in $images) {
        $dim = if ($img.Size -ge 256) { 0 } else { $img.Size }
        $writer.Write([Byte]$dim)
        $writer.Write([Byte]$dim)
        $writer.Write([Byte]0)
        $writer.Write([Byte]0)
        $writer.Write([UInt16]1)
        $writer.Write([UInt16]32)
        $writer.Write([UInt32]$img.Data.Length)
        $writer.Write([UInt32]$offset)
        $offset += $img.Data.Length
    }
    foreach ($img in $images) {
        $writer.Write($img.Data)
    }
    $writer.Dispose()
    $fs.Dispose()
    Write-Host ("  {0,-24} {1}" -f (Split-Path -Leaf $Path), ($Sizes -join '/'))
}

if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

Write-Host "生成 AgentControl 图标 -> $OutputDir"

Save-Png -Size 32  -Path (Join-Path $OutputDir '32x32.png')
Save-Png -Size 128 -Path (Join-Path $OutputDir '128x128.png')
Save-Png -Size 256 -Path (Join-Path $OutputDir '128x128@2x.png')
Save-Png -Size 256 -Path (Join-Path $OutputDir 'icon.png')
Save-Png -Size 50  -Path (Join-Path $OutputDir 'StoreLogo.png')

foreach ($size in @(30, 44, 71, 89, 107, 142, 150, 284, 310)) {
    Save-Png -Size $size -Path (Join-Path $OutputDir ("Square{0}x{0}Logo.png" -f $size))
}

Save-Ico -Sizes @(16, 32, 48, 64, 128, 256) -Path (Join-Path $OutputDir 'icon.ico')

Write-Host '完成。'
