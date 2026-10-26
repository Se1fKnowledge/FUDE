@echo off
rem Starts FUDE in development mode (npm run tauri dev). The desktop shortcut "FUDE"
rem points here. The console stays minimized; closing the FUDE window stops everything.
title FUDE (dev)
cd /d "%~dp0"
if not exist node_modules (
    echo Installing dependencies...
    call npm install || goto :fail
)
call npm run tauri dev || goto :fail
exit /b 0

:fail
echo.
echo FUDE did not start. See the messages above.
pause
exit /b 1
