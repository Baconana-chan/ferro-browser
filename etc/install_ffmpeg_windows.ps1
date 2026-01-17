# Ferro Browser - FFmpeg Installation Script for Windows
# This script installs FFmpeg development libraries using vcpkg

Write-Host "Ferro Browser - FFmpeg Installation" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan

# Check if vcpkg is installed
$vcpkgPath = $env:VCPKG_ROOT
if (-not $vcpkgPath) {
    $vcpkgPath = "C:\vcpkg"
}

if (-not (Test-Path "$vcpkgPath\vcpkg.exe")) {
    Write-Host "vcpkg not found. Installing vcpkg..." -ForegroundColor Yellow
    
    # Clone vcpkg
    git clone https://github.com/Microsoft/vcpkg.git $vcpkgPath
    
    # Bootstrap vcpkg
    Push-Location $vcpkgPath
    .\bootstrap-vcpkg.bat
    Pop-Location
    
    Write-Host "vcpkg installed at $vcpkgPath" -ForegroundColor Green
}

# Set environment variables
$env:VCPKG_ROOT = $vcpkgPath
[System.Environment]::SetEnvironmentVariable("VCPKG_ROOT", $vcpkgPath, "User")

# Install FFmpeg
Write-Host ""
Write-Host "Installing FFmpeg (this may take a while)..." -ForegroundColor Yellow
& "$vcpkgPath\vcpkg.exe" install ffmpeg:x64-windows

# Integrate with system
Write-Host ""
Write-Host "Integrating vcpkg with system..." -ForegroundColor Yellow
& "$vcpkgPath\vcpkg.exe" integrate install

# Set PKG_CONFIG_PATH for ffmpeg
$pkgConfigPath = "$vcpkgPath\installed\x64-windows\lib\pkgconfig"
$env:PKG_CONFIG_PATH = $pkgConfigPath
[System.Environment]::SetEnvironmentVariable("PKG_CONFIG_PATH", $pkgConfigPath, "User")

# Set FFMPEG_DIR
$ffmpegDir = "$vcpkgPath\installed\x64-windows"
$env:FFMPEG_DIR = $ffmpegDir
[System.Environment]::SetEnvironmentVariable("FFMPEG_DIR", $ffmpegDir, "User")

Write-Host ""
Write-Host "FFmpeg installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Environment variables set:" -ForegroundColor Cyan
Write-Host "  VCPKG_ROOT = $vcpkgPath"
Write-Host "  FFMPEG_DIR = $ffmpegDir"
Write-Host "  PKG_CONFIG_PATH = $pkgConfigPath"
Write-Host ""
Write-Host "Please restart your terminal for changes to take effect." -ForegroundColor Yellow
Write-Host "Then run: cargo build -p ferro_media" -ForegroundColor Cyan
