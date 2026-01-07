@echo off
REM Compile SPECTRE CUDA kernels
REM Run this from an x64 Native Tools Command Prompt OR it will set up VS env

where cl.exe >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Setting up Visual Studio environment...
    call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
    if %ERRORLEVEL% NEQ 0 (
        call "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
    )
    if %ERRORLEVEL% NEQ 0 (
        call "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
    )
)

echo Compiling spectre.cu...
nvcc -ptx spectre.cu -o spectre.ptx
if %ERRORLEVEL% EQU 0 (
    echo SUCCESS: spectre.ptx created
) else (
    echo FAILED: Check CUDA installation
    exit /b 1
)
