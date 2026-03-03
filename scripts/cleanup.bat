@echo off
echo Cleaning up Worka processes...

taskkill /F /IM worka.exe 2>nul
if %errorlevel% equ 0 (
    echo [OK] worka.exe terminated
) else (
    echo [SKIP] worka.exe not running
)

taskkill /F /IM node.exe 2>nul
if %errorlevel% equ 0 (
    echo [OK] node.exe terminated
) else (
    echo [SKIP] node.exe not running
)

timeout /t 2 /nobreak >nul

for /f "tokens=5" %%i in ('netstat -ano ^| findstr :1420 ^| findstr LISTENING') do (
    echo [KILL] Process %%i on port 1420
    taskkill /F /PID %%i
)

echo.
echo Cleanup complete!
echo You can now run: npm run dev
