@echo off
chcp 65001 > nul
echo.
echo ========================================
echo   EXE BUILDER - AI Monitor
echo ========================================
echo.

REM Install PyInstaller if needed
pip show pyinstaller > nul 2>&1
if errorlevel 1 (
    echo [INFO] PyInstaller telepitese...
    pip install pyinstaller
)

echo [INFO] EXE keszitese...
pyinstaller --onefile --name "AI_Monitor" --icon=NONE "%~dp0ai_monitor.py"

echo.
echo ========================================
echo   KESZ!
echo   Az EXE itt: dist\AI_Monitor.exe
echo ========================================
pause
