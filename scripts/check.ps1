Write-Host "[1/3] TypeScript type check..." -ForegroundColor Cyan
npx tsc --noEmit
if ($LASTEXITCODE -ne 0) { Write-Host "[FAIL] TypeScript" -ForegroundColor Red; exit 1 }

Write-Host "[2/3] Rust cargo check..." -ForegroundColor Cyan
Push-Location src-tauri
cargo check 2>&1
$rc = $LASTEXITCODE
Pop-Location
if ($rc -ne 0) { Write-Host "[FAIL] Rust" -ForegroundColor Red; exit 1 }

Write-Host "[3/3] Vite build..." -ForegroundColor Cyan
npx vite build
if ($LASTEXITCODE -ne 0) { Write-Host "[FAIL] Vite" -ForegroundColor Red; exit 1 }

Write-Host "[OK] All checks passed" -ForegroundColor Green
