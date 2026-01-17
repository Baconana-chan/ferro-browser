# Setup build environment for Ferro Browser
# Run this script before building with: . .\setup_ffmpeg.ps1

# FFmpeg
$env:FFMPEG_DIR = "C:\ffmpeg71\ffmpeg-n7.1-latest-win64-gpl-shared-7.1"

# LLVM/Clang (for bindgen)
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"

# MozTools (for SpiderMonkey/mozjs build)
$env:MOZTOOLS_PATH = "C:\moztools\moztools-4.0"

# Compilers
$env:CC = "clang-cl"
$env:CXX = "clang-cl"
$env:LD = "lld-link"

# Python
$env:PYTHON3 = "python"

# Add to PATH
$env:PATH = "C:\Program Files\NASM;C:\ffmpeg71\ffmpeg-n7.1-latest-win64-gpl-shared-7.1\bin;$env:PATH"

Write-Host "Ferro Browser build environment configured:" -ForegroundColor Green
Write-Host "  FFMPEG_DIR: $env:FFMPEG_DIR"
Write-Host "  LIBCLANG_PATH: $env:LIBCLANG_PATH"
Write-Host "  MOZTOOLS_PATH: $env:MOZTOOLS_PATH"
Write-Host "  CC/CXX: clang-cl"
Write-Host ""
Write-Host "Build commands:" -ForegroundColor Cyan
Write-Host "  cargo build -p servoshell --features media-ferro  # FFmpeg backend"
Write-Host "  cargo build -p servoshell                         # Dummy backend (no media)"
Write-Host ""
Write-Host "NOTE: GStreamer has been removed. Use media-ferro for video/audio playback." -ForegroundColor Yellow
