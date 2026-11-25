#Requires -Version 5.1
<#
.SYNOPSIS
  Setup otomatis seluruh dependensi GDSS SIEM di Windows (tanpa perlu Run as Administrator).
.EXAMPLE
  powershell -ExecutionPolicy Bypass -File .\setup.ps1
  powershell -ExecutionPolicy Bypass -File .\setup.ps1 -Runtime
#>
param(
    [switch]$Runtime
)

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

function Write-Ok($msg)   { Write-Host "[OK] $msg" -ForegroundColor Green }
function Write-Info($msg) { Write-Host "[..] $msg" -ForegroundColor Yellow }
function Stop-Fail($msg)  { Write-Host "[X]  $msg" -ForegroundColor Red; exit 1 }

# Segarkan PATH dari Environment User + Machine agar instalasi baru langsung terbaca
function Refresh-Path {
    $machine = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $user    = [Environment]::GetEnvironmentVariable("Path", "User")
    $cargo   = Join-Path $env:USERPROFILE ".cargo\bin"
    $npmBin  = Join-Path $env:APPDATA "npm"
    $env:Path = "$cargo;$npmBin;$user;$machine"
}
Refresh-Path

function Find-Python {
    foreach ($candidate in @("python", "python3", "py")) {
        $cmd = Get-Command $candidate -ErrorAction SilentlyContinue
        if (-not $cmd) { continue }
        # Abaikan stub Microsoft Store WindowsApps yang tidak benar-benar terpasang
        if ($cmd.Source -like "*WindowsApps*" -and $candidate -ne "py") {
            try {
                $ver = & $candidate -c "import sys; print(sys.version_info[0])" 2>$null
                if ($LASTEXITCODE -ne 0 -or "$ver".Trim() -ne "3") { continue }
            } catch { continue }
        }
        try {
            if ($candidate -eq "py") {
                $ver = & py -3 -c "import sys; print(f'{sys.version_info[0]}.{sys.version_info[1]}')" 2>$null
                if ($LASTEXITCODE -eq 0 -and $ver) { return @("py", "-3", $ver.Trim()) }
            } else {
                $ver = & $candidate -c "import sys; print(f'{sys.version_info[0]}.{sys.version_info[1]}')" 2>$null
                if ($LASTEXITCODE -eq 0 -and $ver -like "3.*") { return @($candidate, "", $ver.Trim()) }
            }
        } catch { continue }
    }
    return $null
}

# 1. Python 3
$py = Find-Python
if (-not $py -and (Get-Command winget -ErrorAction SilentlyContinue)) {
    Write-Info "Python 3 belum ditemukan, memasang via winget..."
    winget install -e --id Python.Python.3.12 --scope user --accept-package-agreements --accept-source-agreements
    Refresh-Path
    $py = Find-Python
}
if (-not $py) {
    Stop-Fail "Python 3 belum terpasang. Unduh dari https://www.python.org/downloads/ (centang 'Add python.exe to PATH')."
}
Write-Ok "Python $($py[2]) tersedia ($($py[0]))"

# 2. Java >= 11 (wajib untuk Cloud Firestore Emulator)
$hasJava = $false
try {
    $null = & cmd.exe /c "java -version >nul 2>&1"
    if ($LASTEXITCODE -eq 0) { $hasJava = $true }
} catch {}
if (-not $hasJava -and (Get-Command winget -ErrorAction SilentlyContinue)) {
    Write-Info "Java belum ditemukan, memasang Eclipse Temurin JRE 21 via winget..."
    winget install -e --id EclipseAdoptium.Temurin.21.JRE --accept-package-agreements --accept-source-agreements
    Refresh-Path
    $null = & cmd.exe /c "java -version >nul 2>&1"
    if ($LASTEXITCODE -eq 0) { $hasJava = $true }
}
if (-not $hasJava) {
    Stop-Fail "Java (JRE/JDK >= 11, disarankan 21) belum terpasang. Unduh dari https://adoptium.net/"
}
Write-Ok "Java tersedia"

# 3. Node.js & npm (wajib untuk Firebase CLI)
if (-not (Get-Command node -ErrorAction SilentlyContinue) -or -not (Get-Command npm -ErrorAction SilentlyContinue)) {
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        Write-Info "Node.js belum ditemukan, memasang Node.js LTS via winget..."
        winget install -e --id OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements
        Refresh-Path
    }
}
if (-not (Get-Command node -ErrorAction SilentlyContinue) -or -not (Get-Command npm -ErrorAction SilentlyContinue)) {
    Stop-Fail "Node.js & npm belum terpasang. Unduh LTS dari https://nodejs.org/"
}
Write-Ok "Node.js $(& node --version) tersedia"

# 4. Firebase CLI
if (-not (Get-Command firebase -ErrorAction SilentlyContinue)) {
    Write-Info "Memasang Firebase CLI via npm..."
    & cmd.exe /c "npm install -g firebase-tools"
    if ($LASTEXITCODE -ne 0) { Stop-Fail "Gagal memasang firebase-tools." }
    Refresh-Path
}
Write-Ok "Firebase CLI tersedia"

if ($Runtime) {
    Write-Host ""
    Write-Ok "Dependensi runtime Windows siap!"
    exit 0
}

# 5. Rust (untuk kompilasi dari source code)
if (-not (Get-Command rustc -ErrorAction SilentlyContinue) -or -not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Info "Memasang Rust (rustup)..."
    $rustup = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustup -UseBasicParsing
    & $rustup -y --profile minimal
    if ($LASTEXITCODE -ne 0) {
        Stop-Fail "Gagal memasang Rust. Jika terkendala C++ Build Tools, gunakan paket Release Windows (.zip) agar tidak perlu kompilasi!"
    }
    Refresh-Path
}
Write-Ok "Rust $(& rustc --version) tersedia"

# 6. Target WASM
$targets = & rustup target list --installed
if ($targets -notcontains "wasm32-unknown-unknown") {
    Write-Info "Menambahkan target wasm32-unknown-unknown..."
    & rustup target add wasm32-unknown-unknown
}
Write-Ok "Target wasm32-unknown-unknown tersedia"

# 7. cargo-leptos (via cargo-binstall agar tidak perlu kompilasi lama)
if (-not (Get-Command cargo-leptos -ErrorAction SilentlyContinue)) {
    Write-Info "Memasang cargo-leptos..."
    if (-not (Get-Command cargo-binstall -ErrorAction SilentlyContinue)) {
        $zip = Join-Path $env:TEMP "cargo-binstall.zip"
        $dir = Join-Path $env:USERPROFILE ".cargo\bin"
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        Invoke-WebRequest -Uri "https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-pc-windows-msvc.zip" -OutFile $zip -UseBasicParsing
        Expand-Archive -Path $zip -DestinationPath $dir -Force
    }
    & cargo binstall -y cargo-leptos
    if ($LASTEXITCODE -ne 0) {
        & cargo install cargo-leptos --locked
    }
    Refresh-Path
}
Write-Ok "cargo-leptos tersedia"

Write-Host ""
Write-Ok "Semua dependensi siap! Jalankan aplikasi dengan klik dua kali run.bat atau perintah: .\run.ps1"
