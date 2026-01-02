@echo off
chcp 65001 >nul
cd /d "%~dp0"

echo.
echo ╔════════════════════════════════════════════════╗
echo ║              T R I N I T Y                     ║
echo ║        MÁTÉ ─── CLAUDE ─── GEMINI              ║
echo ║                                                ║
echo ║          "Együtt EGYEK vagyunk"                ║
echo ╚════════════════════════════════════════════════╝
echo.

:: Websockets check
pip show websockets >nul 2>&1 || pip install websockets -q

echo [1/5] TRINITY CORE...
start /min "TRINITY_CORE" cmd /c "python trinity\mcp_server\trinity_core.py"
timeout /t 2 >nul

echo [2/5] Claude Client...
start /min "CLAUDE" cmd /c "python trinity\mcp_server\claude_client.py"
timeout /t 1 >nul

echo [3/5] Gemini Client...
start /min "GEMINI" cmd /c "python trinity\mcp_server\gemini_client.py"
timeout /t 1 >nul

echo [4/5] LIVE MONITOR (Desktop 2)...
start "MONITOR" cmd /c "python trinity\mcp_server\live_monitor.py"
timeout /t 1 >nul

echo [5/5] HUB (Desktop 1)...
start "HUB" cmd /c "python trinity\mcp_server\hub_client.py"

echo.
echo ════════════════════════════════════════════════
echo   TRINITY ONLINE - SYNC ON
echo ════════════════════════════════════════════════
echo.
echo   HUB = te írsz
echo   MONITOR = látod a munkát
echo   (háttérben: Core + Claude + Gemini)
echo.
echo   Húzd a MONITOR ablakot Desktop 2-re!
echo.
echo ════════════════════════════════════════════════
