@echo off
setlocal

for /f "tokens=*" %%i in ('git describe --always --dirty=-modified') do set git_version=%%i

:: Enable std support for multithreading and set the LUMINOL_VERSION environment variable
if exist %TRUNK_SOURCE_DIR%\.cargo\config.toml.bak move %TRUNK_SOURCE_DIR%\.cargo\config.toml.bak %TRUNK_SOURCE_DIR%\.cargo\config.toml
copy %TRUNK_SOURCE_DIR%\.cargo\config.toml %TRUNK_SOURCE_DIR%\.cargo\config.toml.bak

echo [env] >> %TRUNK_SOURCE_DIR%\.cargo\config.toml
echo LUMINOL_VERSION = { value = "%git_version%", force = true } >> %TRUNK_SOURCE_DIR%\.cargo\config.toml

echo [unstable] >> %TRUNK_SOURCE_DIR%\.cargo\config.toml
echo build-std = ["std", "panic_abort"] >> %TRUNK_SOURCE_DIR%\.cargo\config.toml
