$ErrorActionPreference = "SilentlyContinue"
Set-Location C:\Users\VIC\ferro-browser
$output = cargo build --package script_bindings --no-default-features --features js-boa 2>&1
$exitCode = $LASTEXITCODE
$errors = $output | Where-Object { $_ -match "^error" }
$result = @()
$result += "=== BUILD RESULT ==="
$result += "Exit code: $exitCode"
$result += "Total output lines: $($output.Count)"
$result += "Error lines: $($errors.Count)"
if ($errors.Count -gt 0) {
    $result += "=== FIRST 10 ERRORS ==="
    $errors | Select-Object -First 10 | ForEach-Object { $result += $_ }
}
$result += "=== LAST 5 OUTPUT LINES ==="
$output | Select-Object -Last 5 | ForEach-Object { $result += $_ }
$result | Out-File -FilePath "C:\Users\VIC\ferro-browser\build_result.txt" -Encoding utf8
