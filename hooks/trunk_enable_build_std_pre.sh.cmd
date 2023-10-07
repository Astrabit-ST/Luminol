@echo off

:: Enable std support for multithreading
if exist %TRUNK_SOURCE_DIR%\.cargo\config.toml.bak mv %TRUNK_SOURCE_DIR%\.cargo\config.toml.bak %TRUNK_SOURCE_DIR%\.cargo\config.toml
cp %TRUNK_SOURCE_DIR%\.cargo\config.toml %TRUNK_SOURCE_DIR%\.cargo\config.toml.bak
echo [unstable] >> %TRUNK_SOURCE_DIR%\.cargo\config.toml
echo build-std = ["std", "panic_abort"] >> %TRUNK_SOURCE_DIR%\.cargo\config.toml
