@echo off
chcp 65001 >nul
title TRINITY HUB - Máté Monitor
cd /d "%~dp0"
python trinity\hub_watcher.py
pause
