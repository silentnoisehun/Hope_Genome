@echo off
chcp 65001 >nul
title TRINITY HUB - Máté
cd /d "%~dp0"
python trinity\mcp_server\hub_client.py
pause
