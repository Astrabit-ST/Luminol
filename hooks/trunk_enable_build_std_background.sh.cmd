@echo off

:: Wait until Trunk errors out or builds successfully, then restore the old Cargo config
:loop
if exist %TRUNK_STAGING_DIR%\luminol.js goto end
tasklist | find trunk.exe >nul
if errorlevel 1 goto end
goto loop
:end
mv %TRUNK_SOURCE_DIR%\.cargo\config.toml.bak %TRUNK_SOURCE_DIR%\.cargo\config.toml
