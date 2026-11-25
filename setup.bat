@echo off
title Setup Dependensi GDSS SIEM
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0setup.ps1" %*
echo.
pause
