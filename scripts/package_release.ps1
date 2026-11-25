#Requires -Version 5.1
<#
.SYNOPSIS
  Mengemas hasil `cargo leptos build --release` di Windows menjadi bundel .zip mandiri
  yang bisa langsung dijalankan teman sekelompok tanpa perlu Rust / C++ Build Tools.
.EXAMPLE
  powershell -ExecutionPolicy Bypass -File .\scripts\package_release.ps1 -Version v1.0.0
#>
param(
    [string]$Version = "v1.0.0"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$Name  = "gdss-siem-$Version-windows-x86_64"
$Stage = Join-Path "dist" $Name
$Zip   = Join-Path "dist" "$Name.zip"
$Sha   = "$Zip.sha256"

if (-not (Test-Path "target\release\gdss-siem.exe") -or -not (Test-Path "target\site\pkg")) {
    Write-Host "[..] Membangun target release Windows..." -ForegroundColor Yellow
    & cargo leptos build --release
    if ($LASTEXITCODE -ne 0) { throw "cargo leptos build --release gagal" }
}

if (Test-Path $Stage) { Remove-Item -Recurse -Force $Stage }
if (Test-Path $Zip)   { Remove-Item -Force $Zip }
if (Test-Path $Sha)   { Remove-Item -Force $Sha }
New-Item -ItemType Directory -Force -Path (Join-Path $Stage "scripts") | Out-Null

Copy-Item "target\release\gdss-siem.exe" (Join-Path $Stage "gdss-siem.exe")
Copy-Item -Recurse "target\site" (Join-Path $Stage "site")
Copy-Item "firebase.json", ".firebaserc", "firestore.rules" $Stage
Copy-Item "scripts\seed_emulator.py" (Join-Path $Stage "scripts\seed_emulator.py")

$StartPs1 = @'
#Requires -Version 5.1
param(
    [switch]$Reset,
    [switch]$Contoh
)

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

function Write-Ok($msg)   { Write-Host "[OK] $msg" -ForegroundColor Green }
function Write-Info($msg) { Write-Host "[..] $msg" -ForegroundColor Yellow }
function Stop-Fail($msg)  { Write-Host "[X]  $msg" -ForegroundColor Red; exit 1 }

function Refresh-Path {
    $machine = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $user    = [Environment]::GetEnvironmentVariable("Path", "User")
    $npmBin  = Join-Path $env:APPDATA "npm"
    $env:Path = "$npmBin;$user;$machine"
}
Refresh-Path

function Test-TcpPort([string]$HostName, [int]$Port) {
    $client = New-Object System.Net.Sockets.TcpClient
    try {
        $iar = $client.BeginConnect($HostName, $Port, $null, $null)
        if (-not $iar.AsyncWaitHandle.WaitOne(500, $false)) { return $false }
        $client.EndConnect($iar)
        return $true
    } catch { return $false } finally { $client.Close() }
}

function Find-PythonCmd {
    foreach ($c in @("python", "python3")) {
        $cmd = Get-Command $c -ErrorAction SilentlyContinue
        if ($cmd -and ($cmd.Source -notlike "*WindowsApps*")) {
            try {
                $v = & $c -c "import sys; print(sys.version_info[0])" 2>$null
                if ($LASTEXITCODE -eq 0 -and "$v".Trim() -eq "3") { return @($c) }
            } catch {}
        }
    }
    if (Get-Command py -ErrorAction SilentlyContinue) {
        try {
            $v = & py -3 -c "import sys; print(sys.version_info[0])" 2>$null
            if ($LASTEXITCODE -eq 0 -and "$v".Trim() -eq "3") { return @("py", "-3") }
        } catch {}
    }
    return $null
}

# 1. Cek Python 3, Java, Node.js, Firebase CLI (bantu pasang otomatis via winget/npm bila belum ada)
$pyCmd = Find-PythonCmd
if (-not $pyCmd -and (Get-Command winget -ErrorAction SilentlyContinue)) {
    Write-Info "Memasang Python 3 via winget..."
    winget install -e --id Python.Python.3.12 --scope user --accept-package-agreements --accept-source-agreements
    Refresh-Path
    $pyCmd = Find-PythonCmd
}
if (-not $pyCmd) { Stop-Fail "Python 3 belum terpasang. Unduh dari https://www.python.org/downloads/ (centang Add to PATH)." }
Write-Ok "Python 3 tersedia"

$hasJava = $false
try { $null = & cmd.exe /c "java -version >nul 2>&1"; if ($LASTEXITCODE -eq 0) { $hasJava = $true } } catch {}
if (-not $hasJava -and (Get-Command winget -ErrorAction SilentlyContinue)) {
    Write-Info "Memasang Eclipse Temurin JRE 21 via winget..."
    winget install -e --id EclipseAdoptium.Temurin.21.JRE --accept-package-agreements --accept-source-agreements
    Refresh-Path
    $null = & cmd.exe /c "java -version >nul 2>&1"
    if ($LASTEXITCODE -eq 0) { $hasJava = $true }
}
if (-not $hasJava) { Stop-Fail "Java (JRE >= 11) belum terpasang. Unduh dari https://adoptium.net/" }
Write-Ok "Java tersedia"

if (-not (Get-Command firebase -ErrorAction SilentlyContinue)) {
    if (-not (Get-Command npm -ErrorAction SilentlyContinue) -and (Get-Command winget -ErrorAction SilentlyContinue)) {
        Write-Info "Memasang Node.js LTS via winget..."
        winget install -e --id OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements
        Refresh-Path
    }
    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        Stop-Fail "Node.js belum terpasang. Unduh LTS dari https://nodejs.org/"
    }
    Write-Info "Memasang Firebase CLI via npm..."
    & cmd.exe /c "npm install -g firebase-tools"
    if ($LASTEXITCODE -ne 0) { Stop-Fail "Gagal memasang firebase-tools." }
    Refresh-Path
}
Write-Ok "Firebase CLI tersedia"

$env:FIREBASE_PROJECT_ID         = "demo-gdss-siem"
$env:FIREBASE_API_KEY            = "demo-key"
$env:FIREBASE_AUTH_EMULATOR_HOST = "127.0.0.1:9099"
$env:FIRESTORE_EMULATOR_HOST     = "127.0.0.1:8080"
$env:LEPTOS_ENV                  = "PROD"
$env:LEPTOS_SITE_ROOT            = "site"
$env:LEPTOS_SITE_PKG_DIR         = "pkg"
if (-not $env:LEPTOS_SITE_ADDR)  { $env:LEPTOS_SITE_ADDR = "127.0.0.1:3000" }

$emuProc = $null
try {
    if ((Test-TcpPort "127.0.0.1" 8080) -and (Test-TcpPort "127.0.0.1" 9099)) {
        Write-Ok "Firebase Emulator sudah aktif"
    } else {
        Write-Info "Menyalakan Firebase Emulator (pertama kali akan mengunduh komponen emulator)..."
        $emuProc = Start-Process -FilePath "cmd.exe" `
            -ArgumentList "/c firebase emulators:start --project demo-gdss-siem > .firebase-emulator.log 2>&1" `
            -WorkingDirectory $PSScriptRoot -WindowStyle Hidden -PassThru
        $ready = $false
        for ($i = 0; $i -lt 120; $i++) {
            if ($emuProc.HasExited) {
                if (Test-Path ".firebase-emulator.log") { Get-Content ".firebase-emulator.log" -Tail 25 }
                Stop-Fail "Firebase Emulator gagal menyala. Cek .firebase-emulator.log"
            }
            if ((Test-TcpPort "127.0.0.1" 8080) -and (Test-TcpPort "127.0.0.1" 9099)) {
                $ready = $true
                break
            }
            Start-Sleep -Seconds 1
        }
        if (-not $ready) { Stop-Fail "Waktu tunggu Firebase Emulator habis." }
        Write-Ok "Firebase Emulator siap"
    }

    $hasUsers = $false
    try {
        $uri = "http://127.0.0.1:8080/v1/projects/demo-gdss-siem/databases/(default)/documents/users?pageSize=1"
        $res = Invoke-RestMethod -Uri $uri -Headers @{ Authorization = "Bearer owner" } -TimeoutSec 3
        if ($res.documents -and $res.documents.Count -gt 0) { $hasUsers = $true }
    } catch {}

    $seedArgs = @()
    if ($Contoh) { $seedArgs = @("--reset", "--contoh") }
    elseif ($Reset) { $seedArgs = @("--reset") }

    if ($seedArgs.Count -gt 0 -or -not $hasUsers) {
        Write-Info "Mengisi data awal GDSS..."
        if ($pyCmd.Count -gt 1) {
            & $pyCmd[0] $pyCmd[1] scripts\seed_emulator.py @seedArgs
        } else {
            & $pyCmd[0] scripts\seed_emulator.py @seedArgs
        }
        if ($LASTEXITCODE -ne 0) { Stop-Fail "Gagal mengisi data awal." }
    } else {
        Write-Ok "Data emulator sudah terisi"
    }

    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host "  GDSS SIEM siap di : http://$($env:LEPTOS_SITE_ADDR)" -ForegroundColor Green
    Write-Host "  Emulator UI       : http://127.0.0.1:4000"
    Write-Host "  Akun Demo:"
    Write-Host "    - CISO          : ciso@gdss-siem.test     / ciso-gdss-2026"
    Write-Host "    - SOC Analyst   : soc@gdss-siem.test      / soc-gdss-2026"
    Write-Host "    - Network Admin : netadmin@gdss-siem.test / netadmin-gdss-2026"
    Write-Host "  Tutup jendela ini atau tekan Ctrl+C untuk berhenti."
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host ""

    Start-Job -ScriptBlock {
        for ($i = 0; $i -lt 20; $i++) {
            Start-Sleep -Milliseconds 500
            try {
                $c = New-Object System.Net.Sockets.TcpClient("127.0.0.1", 3000)
                $c.Close()
                Start-Process "http://127.0.0.1:3000"
                break
            } catch {}
        }
    } | Out-Null

    & .\gdss-siem.exe
} finally {
    if ($emuProc -and -not $emuProc.HasExited) {
        Write-Host ""
        Write-Info "Menghentikan Firebase Emulator..."
        & cmd.exe /c "taskkill /PID $($emuProc.Id) /T /F >nul 2>&1"
    }
}
'@
Set-Content -Path (Join-Path $Stage "start.ps1") -Value $StartPs1 -Encoding UTF8

Set-Content -Path (Join-Path $Stage "start.bat") -Encoding ASCII -Value @"
@echo off
title Ruang Keputusan SIEM - GDSS
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0start.ps1" %*
if errorlevel 1 (
  echo.
  pause
)
"@

Set-Content -Path (Join-Path $Stage "reset-data.bat") -Encoding ASCII -Value @"
@echo off
title Ruang Keputusan SIEM - Reset Data Awal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0start.ps1" -Reset
if errorlevel 1 (
  echo.
  pause
)
"@

Set-Content -Path (Join-Path $Stage "demo-siap-voting.bat") -Encoding ASCII -Value @"
@echo off
title Ruang Keputusan SIEM - Demo Siap Voting CISO
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0start.ps1" -Contoh
if errorlevel 1 (
  echo.
  pause
)
"@

Compress-Archive -Path $Stage -DestinationPath $Zip -Force
$hash = (Get-FileHash -Algorithm SHA256 $Zip).Hash.ToLower()
Set-Content -Path $Sha -Encoding ASCII -Value "$hash  $Name.zip"

Write-Host "[OK] Paket rilis Windows dibuat: $Zip" -ForegroundColor Green
Get-Content $Sha
