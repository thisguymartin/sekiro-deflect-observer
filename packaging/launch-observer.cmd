@echo off
setlocal
where me3.exe >nul 2>nul
if errorlevel 1 (
    echo me3 was not found on PATH.
    echo Install me3 from https://github.com/garyttierney/me3/releases
    echo Then open a new terminal or double-click observer.me3.
    pause
    exit /b 1
)
if not exist "%~dp0sekiro_deflect_observer.dll" (
    echo sekiro_deflect_observer.dll is missing. Extract the entire package first.
    pause
    exit /b 1
)
echo Close Sekiro before launching this profile. Keep Steam running.
me3.exe launch --game sekiro --profile "%~dp0observer.me3"
set "OBSERVER_RESULT=%ERRORLEVEL%"
if not "%OBSERVER_RESULT%"=="0" (
    echo Launch failed. Keep this output for the bug report.
    pause
)
exit /b %OBSERVER_RESULT%
