@echo off
setlocal

for /f "tokens=*" %%i in ('git describe --always --dirty=-modified') do set git_version=%%i

:: Print build information to buildinfo.json in the root directory of the output folder
:: You can change the "epoch" if you need to make backwards-incompatible changes to the build info
echo {"epoch":0,"rev":"%git_version%","profile":"%TRUNK_PROFILE%"} > %TRUNK_STAGING_DIR%\buildinfo.json

:: Enable std support for multithreading and set the LUMINOL_VERSION environment variable
if exist .cargo\config.toml.bak move .cargo\config.toml.bak .cargo\config.toml
copy .cargo\config.toml .cargo\config.toml.bak

echo LUMINOL_VERSION = { value = "%git_version%", force = true } >> .cargo\config.toml

echo [unstable] >> .cargo\config.toml
echo build-std = ["std", "panic_abort"] >> .cargo\config.toml
