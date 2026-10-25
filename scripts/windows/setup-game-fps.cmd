@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0setup-game-fps.ps1" %*
set "fps_result=%errorlevel%"
pause
exit /b %fps_result%
