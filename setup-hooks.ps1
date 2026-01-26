# ============================================================================
# Setup Git Hooks - PowerShell Version
# ============================================================================
# Run this script once to install Git hooks for the project
# Usage: .\setup-hooks.ps1
# ============================================================================

Write-Host "Setting up Git hooks..." -ForegroundColor Cyan

# Create hooks directory if it doesn't exist
if (-not (Test-Path ".git\hooks")) {
    New-Item -ItemType Directory -Path ".git\hooks" -Force | Out-Null
}

# Copy pre-commit hook
if (Test-Path "hooks\pre-commit") {
    Copy-Item "hooks\pre-commit" ".git\hooks\pre-commit" -Force
    Write-Host "Pre-commit hook installed" -ForegroundColor Green
} else {
    Write-Host "ERROR: hooks\pre-commit not found" -ForegroundColor Red
    exit 1
}

# Copy pre-push hook if it exists
if (Test-Path "hooks\pre-push") {
    Copy-Item "hooks\pre-push" ".git\hooks\pre-push" -Force
    Write-Host "Pre-push hook installed" -ForegroundColor Green
}

Write-Host ""
Write-Host "Git hooks setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Hooks will now run automatically:" -ForegroundColor Yellow
Write-Host "  - pre-commit: Runs before each commit" -ForegroundColor White
Write-Host "  - Checks: lint, format, test" -ForegroundColor White
Write-Host ""
Write-Host "To skip hooks temporarily:" -ForegroundColor Yellow
Write-Host "  git commit --no-verify" -ForegroundColor White
Write-Host ""
Write-Host "To run checks manually:" -ForegroundColor Yellow
Write-Host "  cd monitor\backend" -ForegroundColor White
Write-Host "  cargo fmt" -ForegroundColor White
Write-Host "  cargo clippy" -ForegroundColor White
Write-Host "  cargo test" -ForegroundColor White
