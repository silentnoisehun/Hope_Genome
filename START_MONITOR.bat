@echo off
chcp 65001 >nul
title TRINITY LIVE MONITOR
cd /d "%~dp0"
python trinity\mcp_server\live_monitor.py
pause
