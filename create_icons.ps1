Add-Type -AssemblyName System.Drawing

# Ensure directories exist
$resourcesDir = "resources"
$iconsDir = Join-Path $resourcesDir "icons"

if (-not (Test-Path $resourcesDir)) {
    New-Item -Path $resourcesDir -ItemType Directory | Out-Null
}

if (-not (Test-Path $iconsDir)) {
    New-Item -Path $iconsDir -ItemType Directory | Out-Null
}

# Create a "stopped" icon (red)
$stoppedIcon = New-Object System.Drawing.Bitmap 32, 32
$graphics = [System.Drawing.Graphics]::FromImage($stoppedIcon)
$graphics.Clear([System.Drawing.Color]::Transparent)

$redBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::Red)
$graphics.FillEllipse($redBrush, 4, 4, 24, 24)

$whitePen = New-Object System.Drawing.Pen([System.Drawing.Color]::White, 2)
$graphics.DrawLine($whitePen, 12, 12, 20, 20)
$graphics.DrawLine($whitePen, 12, 20, 20, 12)

$icon = [System.Drawing.Icon]::FromHandle($stoppedIcon.GetHicon())
$iconPath = Join-Path $iconsDir "syncthing_stopped.ico"
$fileStream = New-Object System.IO.FileStream($iconPath, [System.IO.FileMode]::Create)
$icon.Save($fileStream)
$fileStream.Close()
$icon.Dispose()
$stoppedIcon.Dispose()

Write-Host "Created stopped icon at $iconPath"

# Create a "running" icon (green)
$runningIcon = New-Object System.Drawing.Bitmap 32, 32
$graphics = [System.Drawing.Graphics]::FromImage($runningIcon)
$graphics.Clear([System.Drawing.Color]::Transparent)

$greenBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::Green)
$graphics.FillEllipse($greenBrush, 4, 4, 24, 24)

$whitePen = New-Object System.Drawing.Pen([System.Drawing.Color]::White, 2)
$graphics.DrawLine($whitePen, 10, 16, 14, 20)
$graphics.DrawLine($whitePen, 14, 20, 22, 12)

$icon = [System.Drawing.Icon]::FromHandle($runningIcon.GetHicon())
$iconPath = Join-Path $iconsDir "syncthing_running.ico"
$fileStream = New-Object System.IO.FileStream($iconPath, [System.IO.FileMode]::Create)
$icon.Save($fileStream)
$fileStream.Close()
$icon.Dispose()
$runningIcon.Dispose()

Write-Host "Created running icon at $iconPath"

# Create a generic app icon
$appIcon = New-Object System.Drawing.Bitmap 32, 32
$graphics = [System.Drawing.Graphics]::FromImage($appIcon)
$graphics.Clear([System.Drawing.Color]::Transparent)

$blueBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::RoyalBlue)
$graphics.FillEllipse($blueBrush, 4, 4, 24, 24)

$whitePen = New-Object System.Drawing.Pen([System.Drawing.Color]::White, 2)
$graphics.DrawLine($whitePen, 12, 12, 20, 12)
$graphics.DrawLine($whitePen, 12, 16, 20, 16)
$graphics.DrawLine($whitePen, 12, 20, 20, 20)

$icon = [System.Drawing.Icon]::FromHandle($appIcon.GetHicon())
$iconPath = Join-Path $iconsDir "syncthingers.ico"
$fileStream = New-Object System.IO.FileStream($iconPath, [System.IO.FileMode]::Create)
$icon.Save($fileStream)
$fileStream.Close()
$icon.Dispose()
$appIcon.Dispose()

Write-Host "Created app icon at $iconPath"

Write-Host "All icons created successfully!" 