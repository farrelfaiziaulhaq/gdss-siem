#Requires -Version 5.1
<#
.SYNOPSIS
  Runner 1 perintah untuk GDSS SIEM di Windows (Emulator + Seed + Server Web).
.EXAMPLE
  .\run.ps1
  .\run.ps1 -Reset
  .\run.ps1 -Contoh
  .\run.ps1 -Release
#>
param(
    [switch]$Reset,
    [switch]$Contoh,
    [switch]$Release
)

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

function Write-Ok($msg)   { Write-Host "[OK] $msg" -ForegroundColor Green }
function Write-Info($msg) { Write-Host "[..] $msg" -ForegroundColor Yellow }
function Stop-Fail($msg)  { Write-Host "[X]  $msg" -ForegroundColor Red; exit 1 }

$cargo  = Join-Path $env:USERPROFILE ".cargo\bin"
$npmBin = Join-Path $env:APPDATA "npm"
$env:Path = "$cargo;$npmBin;$env:Path"

function Test-TcpPort([string]$HostName, [int]$Port) {
    $client = New-Object System.Net.Sockets.TcpClient
    try {
        $iar = $client.BeginConnect($HostName, $Port, $null, $null)
        if (-not $iar.AsyncWaitHandle.WaitOne(500, $false)) { return $false }
        $client.EndConnect($iar)
        return $true
    } catch {
        return $false
    } finally {
        $client.Close()
    }
}

function Invoke-Seed([string[]]$SeedArgs) {
    foreach ($candidate in @("python", "python3")) {
        $cmd = Get-Command $candidate -ErrorAction SilentlyContinue
        if ($cmd -and ($cmd.Source -notlike "*WindowsApps*")) {
            & $candidate scripts\seed_emulator.py @SeedArgs
            if ($LASTEXITCODE -eq 0) { return }
        }
    }
    if (Get-Command py -ErrorAction SilentlyContinue) {
        & py -3 scripts\seed_emulator.py @SeedArgs
        if ($LASTEXITCODE -eq 0) { return }
    }
    & python scripts\seed_emulator.py @SeedArgs
    if ($LASTEXITCODE -ne 0) { Stop-Fail "Gagal menjalankan scripts\seed_emulator.py" }
}

# 1. Pastikan dependensi tersedia
$needSetup = (-not (Get-Command firebase -ErrorAction SilentlyContinue)) -or
             (-not (Get-Command java -ErrorAction SilentlyContinue))
if (-not $Release -and -not (Get-Command cargo-leptos -ErrorAction SilentlyContinue)) {
    $needSetup = $true
}
if ($needSetup) {
    Write-Info "Dependensi belum lengkap, menjalankan setup.ps1..."
    if ($Release -and (Test-Path "target\release\gdss-siem.exe")) {
        & powershell -NoProfile -ExecutionPolicy Bypass -File "$PSScriptRoot\setup.ps1" -Runtime
    } else {
        & powershell -NoProfile -ExecutionPolicy Bypass -File "$PSScriptRoot\setup.ps1"
    }
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

if (-not $env:FIREBASE_PROJECT_ID)         { $env:FIREBASE_PROJECT_ID = "demo-gdss-siem" }
if (-not $env:FIREBASE_API_KEY)            { $env:FIREBASE_API_KEY = "demo-key" }
if (-not $env:FIREBASE_AUTH_EMULATOR_HOST) { $env:FIREBASE_AUTH_EMULATOR_HOST = "127.0.0.1:9099" }
if (-not $env:FIRESTORE_EMULATOR_HOST)     { $env:FIRESTORE_EMULATOR_HOST = "127.0.0.1:8080" }
if (-not $env:LEPTOS_SITE_ADDR)            { $env:LEPTOS_SITE_ADDR = "127.0.0.1:3000" }

$emuProc = $null
try {
    # 2. Nyalakan Firebase Emulator bila belum jalan
    if ((Test-TcpPort "127.0.0.1" 8080) -and (Test-TcpPort "127.0.0.1" 9099)) {
        Write-Ok "Firebase Emulator sudah aktif di 127.0.0.1:8080 & 127.0.0.1:9099"
    } else {
        Write-Info "Menyalakan Firebase Emulator (Auth :9099, Firestore :8080, UI :4000)..."
        $emuProc = Start-Process -FilePath "cmd.exe" `
            -ArgumentList "/c firebase emulators:start --project $($env:FIREBASE_PROJECT_ID) > .firebase-emulator.log 2>&1" `
            -WorkingDirectory $PSScriptRoot -WindowStyle Hidden -PassThru
        $ready = $false
        for ($i = 0; $i -lt 90; $i++) {
            if ($emuProc.HasExited) {
                if (Test-Path ".firebase-emulator.log") { Get-Content ".firebase-emulator.log" -Tail 25 }
                Stop-Fail "Firebase Emulator berhenti sebelum siap."
            }
            if ((Test-TcpPort "127.0.0.1" 8080) -and (Test-TcpPort "127.0.0.1" 9099)) {
                $ready = $true
                break
            }
            Start-Sleep -Seconds 1
        }
        if (-not $ready) {
            if (Test-Path ".firebase-emulator.log") { Get-Content ".firebase-emulator.log" -Tail 25 }
            Stop-Fail "Waktu tunggu Firebase Emulator habis."
        }
        Write-Ok "Firebase Emulator siap"
    }

    # 3. Seed otomatis bila koleksi users masih kosong atau diminta -Reset / -Contoh
    $hasUsers = $false
    try {
        $uri = "http://$($env:FIRESTORE_EMULATOR_HOST)/v1/projects/$($env:FIREBASE_PROJECT_ID)/databases/(default)/documents/users?pageSize=1"
        $res = Invoke-RestMethod -Uri $uri -Headers @{ Authorization = "Bearer owner" } -TimeoutSec 3
        if ($res.documents -and $res.documents.Count -gt 0) { $hasUsers = $true }
    } catch {}

    $seedArgs = @()
    if ($Contoh) { $seedArgs = @("--reset", "--contoh") }
    elseif ($Reset) { $seedArgs = @("--reset") }

    if ($seedArgs.Count -gt 0 -or -not $hasUsers) {
        Write-Info "Mengisi data awal GDSS ke emulator..."
        Invoke-Seed $seedArgs
    } else {
        Write-Ok "Data emulator sudah terisi (gunakan .\run.ps1 -Reset untuk mereset)"
    }

    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host "  Aplikasi Web GDSS : http://$($env:LEPTOS_SITE_ADDR)" -ForegroundColor Green
    Write-Host "  Emulator UI       : http://127.0.0.1:4000"
    Write-Host "  Akun Demo:"
    Write-Host "    - CISO          : ciso@gdss-siem.test     / ciso-gdss-2026"
    Write-Host "    - SOC Analyst   : soc@gdss-siem.test      / soc-gdss-2026"
    Write-Host "    - Network Admin : netadmin@gdss-siem.test / netadmin-gdss-2026"
    Write-Host "  Tekan Ctrl+C untuk menghentikan server & emulator."
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host ""

    # 4. Jalankan server
    if ($Release) {
        if (-not (Test-Path "target\release\gdss-siem.exe") -or -not (Test-Path "target\site\pkg")) {
            Write-Info "Membangun versi release..."
            & cargo leptos build --release
            if ($LASTEXITCODE -ne 0) { Stop-Fail "Build release gagal." }
        }
        $env:LEPTOS_ENV = "PROD"
        $env:LEPTOS_SITE_ROOT = "target/site"
        Start-Job -ScriptBlock {
            for ($i = 0; $i -lt 30; $i++) {
                Start-Sleep -Seconds 1
                try {
                    $c = New-Object System.Net.Sockets.TcpClient("127.0.0.1", 3000)
                    $c.Close()
                    Start-Process "http://127.0.0.1:3000"
                    break
                } catch {}
            }
        } | Out-Null
        & .\target\release\gdss-siem.exe
    } else {
        & cargo leptos serve
    }
} finally {
    if ($emuProc -and -not $emuProc.HasExited) {
        Write-Host ""
        Write-Info "Menghentikan Firebase Emulator..."
        & cmd.exe /c "taskkill /PID $($emuProc.Id) /T /F >nul 2>&1"
    }
}
