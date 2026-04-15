@echo off
:: ============================================================
::  Velaris Lead-Gen Bot — Launcher
::  Doble clic para ejecutar. Lanza PowerShell sin bloqueos.
:: ============================================================
title Velaris Lead-Gen Bot

:: Limpiar pantalla
cls

:: Ejecutar el script principal de PowerShell esquivando restricciones
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0run.ps1"

:: Mantener la consola abierta obligatoriamente al finalizar o en caso de error
echo.
echo ============================================================
echo  Ejecucion terminada. Puedes cerrar esta ventana.
echo ============================================================
pause
