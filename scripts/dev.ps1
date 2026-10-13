# Run the Pharmacy server and desktop app on this Windows machine.
#
# Development only. On first run it creates the database, loads the stand-in for
# MEDITRINO's medicine_catalog, and applies the pharmacy migration. The desktop
# app signs in with the debug-only developer token, so no password is needed.
#
# Requirements: cargo (with MinGW on PATH for the windows-gnu toolchain) and a
# PostgreSQL install. By default it uses the portable install next to the repo
# (..\tools\postgresql) on port 5433.
#
# Usage (from anywhere):
#   powershell -ExecutionPolicy Bypass -File scripts\dev.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\dev.ps1 -ServerOnly
param(
    [switch]$ServerOnly,
    [int]$Port = 3001,
    [string]$PgHome = $(if ($env:PHARMACY_PG_HOME) { $env:PHARMACY_PG_HOME } else { Join-Path $PSScriptRoot '..\..\tools\postgresql' }),
    [int]$PgPort = $(if ($env:PHARMACY_PG_PORT) { [int]$env:PHARMACY_PG_PORT } else { 5433 }),
    [string]$DbName = 'pharmacy_local'
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$PgHome = (Resolve-Path $PgHome).Path
$pgBin = Join-Path $PgHome 'bin'
$pgData = Join-Path $PgHome 'data'
$psql = Join-Path $pgBin 'psql.exe'
$env:PGPASSWORD = 'postgres'
$adminUrl = "postgresql://postgres@localhost:$PgPort/postgres"
$dbUrl = "postgresql://postgres@localhost:$PgPort/$DbName"

function Invoke-Psql([string]$url, [string[]]$arguments) {
    $output = & $psql $url -v ON_ERROR_STOP=1 -q @arguments
    if ($LASTEXITCODE -ne 0) { throw "psql failed: $arguments" }
    $output
}

Write-Host "==> PostgreSQL ($PgHome, port $PgPort)"
& (Join-Path $pgBin 'pg_ctl.exe') -D $pgData status *> $null
if ($LASTEXITCODE -ne 0) {
    $log = Join-Path $PgHome 'log\postgres.log'
    New-Item -ItemType Directory -Force (Split-Path $log) | Out-Null
    & (Join-Path $pgBin 'pg_ctl.exe') -D $pgData -l $log -w start | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "PostgreSQL did not start; see $log" }
    Write-Host '    started'
}

if ((Invoke-Psql $adminUrl @('-tAc', "SELECT 1 FROM pg_database WHERE datname = '$DbName'")) -ne '1') {
    Invoke-Psql $adminUrl @('-c', "CREATE DATABASE `"$DbName`"") | Out-Null
    Write-Host "    created database $DbName"
}
# The pharmacy migration depends on medicine_catalog, so the stand-in goes first.
if ((Invoke-Psql $dbUrl @('-tAc', "SELECT to_regclass('medicine_catalog') IS NOT NULL")) -ne 't') {
    Invoke-Psql $dbUrl @('-f', (Join-Path $repo 'crates\server\tests\fixtures\medicine_catalog.sql')) | Out-Null
    Write-Host '    loaded medicine_catalog stand-in (development only)'
}
if ((Invoke-Psql $dbUrl @('-tAc', "SELECT to_regclass('pharmacies') IS NOT NULL")) -ne 't') {
    Invoke-Psql $dbUrl @('-f', (Join-Path $repo 'migrations\postgres\20261009_pharmacy_auth_and_sync.sql')) | Out-Null
    Write-Host '    applied pharmacy migration'
}

Write-Host '==> Building'
Push-Location $repo
try {
    $packages = if ($ServerOnly) { @('-p', 'server') } else { @('-p', 'server', '-p', 'frontend') }
    cargo build @packages
    if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
} finally { Pop-Location }

# A per-machine developer token, kept under target\ so git never sees it.
$tokenFile = Join-Path $repo 'target\dev-token.txt'
if (-not (Test-Path $tokenFile)) {
    $bytes = New-Object byte[] 32
    [Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)
    [Convert]::ToBase64String($bytes).TrimEnd('=').Replace('+', '-').Replace('/', '_') | Set-Content -Encoding ascii -NoNewline $tokenFile
}
$env:PHARMACY_DEV_TOKEN = (Get-Content $tokenFile -Raw).Trim()
$env:DATABASE_URL = "postgresql://postgres:postgres@localhost:$PgPort/$DbName"
$env:ADDR = "127.0.0.1:$Port"
$env:PHARMACY_SECURE_COOKIES = 'false'
$env:PHARMACY_API_BASE_URL = "http://127.0.0.1:$Port"

$serverExe = Join-Path $repo 'target\debug\server.exe'
if ($ServerOnly) {
    Write-Host "==> Server on http://127.0.0.1:$Port (Ctrl+C to stop)"
    & $serverExe
    exit $LASTEXITCODE
}

$serverLog = Join-Path $repo 'target\server.log'
Write-Host "==> Server on http://127.0.0.1:$Port (log: $serverLog)"
$server = Start-Process -FilePath $serverExe -WorkingDirectory $repo -NoNewWindow -PassThru `
    -RedirectStandardOutput $serverLog -RedirectStandardError "$serverLog.err"
try {
    $ready = $false
    for ($i = 0; $i -lt 50 -and -not $server.HasExited; $i++) {
        try { Invoke-WebRequest "http://127.0.0.1:$Port/login" -UseBasicParsing -TimeoutSec 2 | Out-Null; $ready = $true; break }
        catch { Start-Sleep -Milliseconds 200 }
    }
    if (-not $ready) { throw "Server did not start; see $serverLog.err" }
    Write-Host '==> Desktop app (close the window to stop everything)'
    & (Join-Path $repo 'target\debug\frontend.exe')
} finally {
    if (-not $server.HasExited) { Stop-Process -Id $server.Id }
}
