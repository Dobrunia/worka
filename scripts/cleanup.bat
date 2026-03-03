@echo off
echo Cleaning up Worka processes...

:: 1. Kill worka.exe (the compiled Rust binary)
taskkill /F /IM worka.exe 2>nul
if %errorlevel% equ 0 (
    echo [OK] worka.exe terminated
) else (
    echo [SKIP] worka.exe not running
)

:: 2. Kill cargo.exe (may still be compiling in the background)
taskkill /F /IM cargo.exe 2>nul
if %errorlevel% equ 0 (
    echo [OK] cargo.exe terminated
) else (
    echo [SKIP] cargo.exe not running
)

:: 3. Kill processes listening on port 1420 (Vite dev server)
for /f "tokens=5" %%i in ('netstat -ano 2^>nul ^| findstr ":1420 " ^| findstr LISTENING') do (
    echo [KILL] port 1420 — PID %%i
    taskkill /F /PID %%i 2>nul
)

:: 4. Kill processes listening on port 1421 (Vite HMR websocket)
for /f "tokens=5" %%i in ('netstat -ano 2^>nul ^| findstr ":1421 " ^| findstr LISTENING') do (
    echo [KILL] port 1421 — PID %%i
    taskkill /F /PID %%i 2>nul
)

:: Short pause so OS releases ports before next launch
timeout /t 1 /nobreak >nul

echo.
echo Cleanup complete. Starting dev...
