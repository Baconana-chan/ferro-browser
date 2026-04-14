$ErrorActionPreference = "SilentlyContinue"
Set-Location C:\Users\VIC\ferro-browser
$output = cargo build --package script_bindings --no-default-features --features js-boa 2>&1
$exitCode = $LASTEXITCODE
$errors = $output | Where-Object { $_ -match "^error" }
$warnings = $output | Where-Object { $_ -match "warning\[" }
Write-Output "=== BUILD RESULT ==="
Write-Output "Exit code: $exitCode"
Write-Output "Error lines: $($errors.Count)"
Write-Output "Warning lines: $($warnings.Count)"
if ($errors.Count -gt 0) {
    Write-Output "=== FIRST 10 ERRORS ==="
    $errors | Select-Object -First 10
}
Write-Output "=== LAST 5 OUTPUT LINES ==="
$output | Select-Object -Last 5
