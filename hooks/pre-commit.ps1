# ============================================================================
# Smart Patient Room Monitor - Pre-commit Hook (PowerShell)
# ============================================================================
# This hook runs before each commit to ensure code quality.
# Installation: .\setup-hooks.ps1
# ============================================================================

$ErrorActionPreference = "Continue"

Write-Host "Running pre-commit checks..." -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan

# Track if any check fails
$FAILED = 0

# ----------------------------------------------------------------------------
# 1. Rust Format Check
# ----------------------------------------------------------------------------
Write-Host "`n[1/4] Checking Rust formatting..." -ForegroundColor Yellow
Push-Location monitor\backend
$null = cargo fmt --all -- --check 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "OK Formatting OK" -ForegroundColor Green
} else {
    Write-Host "X Formatting issues found!" -ForegroundColor Red
    Write-Host "  Run cargo fmt to fix"
    $FAILED = 1
}

# ----------------------------------------------------------------------------
# 2. Rust Linting (Clippy)
# ----------------------------------------------------------------------------
Write-Host "`n[2/4] Running Clippy linter..." -ForegroundColor Yellow
$null = cargo clippy --all-targets --all-features -- -D warnings 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "OK Clippy OK" -ForegroundColor Green
} else {
    Write-Host "X Clippy found issues!" -ForegroundColor Red
    $FAILED = 1
}

# ----------------------------------------------------------------------------
# 3. Rust Tests
# ----------------------------------------------------------------------------
Write-Host "`n[3/4] Running Rust tests..." -ForegroundColor Yellow
$null = cargo test 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "OK Tests passed" -ForegroundColor Green
} else {
    Write-Host "X Tests failed!" -ForegroundColor Red
    $FAILED = 1
}
Pop-Location

# ----------------------------------------------------------------------------
# 4. Check for secrets
# ----------------------------------------------------------------------------
Write-Host "`n[4/4] Checking for hardcoded secrets..." -ForegroundColor Yellow

$stagedDiff = git diff --cached
$SECRET_FOUND = $false

# Look for real hardcoded secrets (not env var fallbacks or test values)
if ($stagedDiff -match 'password\s*=\s*"(?!.*\$\{)[a-zA-Z0-9!@#]{8,}"' -and
    $stagedDiff -notmatch 'postgres|development_only|test_secret|ci\.yml|CODESPACES|\.example') {
    $SECRET_FOUND = $true
}

if (-not $SECRET_FOUND) {
    Write-Host "OK No hardcoded secrets detected" -ForegroundColor Green
} else {
    Write-Host "X Possible hardcoded secrets found!" -ForegroundColor Red
    $FAILED = 1
}

# ----------------------------------------------------------------------------
# Final Result
# ----------------------------------------------------------------------------
Write-Host ""
Write-Host "================================" -ForegroundColor Cyan
if ($FAILED -eq 0) {
    Write-Host "All pre-commit checks passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "Pre-commit checks failed!" -ForegroundColor Red
    exit 1
}
