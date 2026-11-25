@echo off
title Ruang Keputusan SIEM - GDSS
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0run.ps1" %*
if errorlevel 1 (
  echo.
  pause
)
