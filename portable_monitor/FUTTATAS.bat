@echo off
chcp 65001 > nul
title AI Monitor - Hope Genome
echo.
echo ========================================
echo   AI CONVERSATION MONITOR
echo   Portable Edition
echo ========================================
echo.

REM Check if Python is available
python --version > nul 2>&1
if errorlevel 1 (
    echo [HIBA] Python nincs telepitve!
    echo Telepitsd: https://python.org/downloads
    pause
    exit /b 1
)

REM Install pyperclip if needed
pip show pyperclip > nul 2>&1
if errorlevel 1 (
    echo [INFO] pyperclip telepitese...
    pip install pyperclip
)

REM Run the monitor
python "%~dp0ai_monitor.py"

pause
