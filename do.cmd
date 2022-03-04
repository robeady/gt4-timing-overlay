call :%~1
@ exit /b %errorlevel%

:inject
Injector.exe -n pcsx2.exe -i target\i686-pc-windows-msvc\release\timing_lib.dll
@ exit /b %errorlevel%

:eject
Injector.exe -n pcsx2.exe -e target\i686-pc-windows-msvc\release\timing_lib.dll
@ exit /b %errorlevel%

:build
cargo build --release
@ exit /b %errorlevel%

:run
set RUST_BACKTRACE=1
cargo run --release
@ exit /b %errorlevel%
