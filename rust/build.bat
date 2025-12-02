@echo off
echo 編譯肥米輸入法 Rust 版本...
cargo build --release
if %ERRORLEVEL% EQU 0 (
    echo.
    echo 編譯成功！
    echo 執行檔位置: target\release\uclliu.exe
) else (
    echo.
    echo 編譯失敗！
    pause
)

