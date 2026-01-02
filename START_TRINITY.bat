@echo off
chcp 65001 >nul
title TRINITY STARTER
cd /d "%~dp0"

echo.
echo ╔════════════════════════════════════════════════╗
echo ║           T R I N I T Y   S T A R T            ║
echo ╚════════════════════════════════════════════════╝
echo.

:: Websockets check
pip show websockets >nul 2>&1 || pip install websockets -q

echo [1/4] TRINITY CORE indítása (háttér)...
start /min "TRINITY_CORE" cmd /c "python trinity\mcp_server\trinity_core.py"
timeout /t 2 >nul

echo [2/4] Claude Client indítása (háttér)...
start /min "CLAUDE_CLIENT" cmd /c "python trinity\mcp_server\claude_client.py"
timeout /t 1 >nul

echo [3/4] Gemini Client indítása (háttér)...
start /min "GEMINI_CLIENT" cmd /c "python trinity\mcp_server\gemini_client.py"
timeout /t 1 >nul

echo [4/4] Rendszer kész!
echo.
echo ════════════════════════════════════════════════
echo.
echo   Most indítsd el külön ablakokban:
echo.
echo   DESKTOP 1: START_HUB.bat (te írsz ide)
echo   DESKTOP 2: START_MONITOR.bat (itt látod a munkát)
echo.
echo ════════════════════════════════════════════════
echo.
pause
