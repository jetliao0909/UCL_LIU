# Windows 版本兼容性說明

## 支援的 Windows 版本

本程式理論上支援以下 Windows 版本：

- ✅ **Windows 10** - 完全支援
- ✅ **Windows 11** - 完全支援
- ⚠️ **Windows 7** - 理論支援，但未充分測試

## 使用的 Windows API 兼容性分析

### 核心 API

1. **SetWindowsHookExW** (WH_KEYBOARD_LL)
   - 最低版本：Windows NT 4.0
   - Windows 7/10/11：✅ 完全支援

2. **GetMessageW / TranslateMessage / DispatchMessageW**
   - 最低版本：Windows 95
   - Windows 7/10/11：✅ 完全支援

3. **PostQuitMessage**
   - 最低版本：Windows 95
   - Windows 7/10/11：✅ 完全支援

4. **SendInput**
   - 最低版本：Windows XP
   - Windows 7/10/11：✅ 完全支援

5. **Shell_NotifyIcon** (系統托盤)
   - 最低版本：Windows 95
   - Windows 7/10/11：✅ 完全支援

6. **WM_COMMAND** (消息處理)
   - 最低版本：Windows 95
   - Windows 7/10/11：✅ 完全支援

### 依賴庫兼容性

1. **windows-rs 0.52**
   - 官方支援：Windows 7 SP1+
   - 實際測試：Windows 10/11 完全正常
   - Windows 7：理論支援，但可能需要較新的 Visual C++ Runtime

2. **tray-icon 0.10**
   - 基於 Windows API，理論上支援 Windows 7+
   - 實際測試：Windows 10/11 完全正常
   - Windows 7：可能需要額外測試

3. **arboard 3.2** (剪貼簿操作)
   - 使用標準 Windows API
   - Windows 7/10/11：✅ 完全支援

## 編譯要求

### Windows 7
- Rust 1.70+
- Windows SDK 7.1 或更高版本
- Visual Studio Build Tools 2015 或更高版本
- Visual C++ Redistributable（運行時需要）

### Windows 10/11
- Rust 1.70+
- Windows SDK 10.0 或更高版本
- Visual Studio Build Tools 2019 或更高版本（推薦 2022）

## 已知限制

1. **Windows 7**
   - ⚠️ 已停止支援（2020年1月），微軟不再提供安全更新
   - ⚠️ 可能需要較新的 Visual C++ Runtime（建議安裝 2015-2022 版本）
   - ⚠️ 未進行充分測試，可能存在未知問題

2. **Windows 8/8.1**
   - ⚠️ 未測試，理論上應該支援（使用相同的 API）

3. **系統托盤功能**
   - Windows 7/10/11 都支援，但行為可能略有不同
   - Windows 11 的系統托盤圖示顯示方式有所改變

## 建議

1. **生產環境**：建議使用 Windows 10 或 Windows 11
2. **Windows 7 用戶**：
   - 確保已安裝最新的 Visual C++ Redistributable
   - 如果遇到問題，請回報具體錯誤訊息
3. **測試**：如果需要在 Windows 7 上運行，建議先進行充分測試

## 測試狀態

- ✅ Windows 10：已測試，完全正常
- ✅ Windows 11：已測試，完全正常
- ⚠️ Windows 7：未充分測試，理論支援

## 相關資源

- [Windows API 文檔](https://docs.microsoft.com/en-us/windows/win32/api/)
- [windows-rs 文檔](https://github.com/microsoft/windows-rs)
- [Visual C++ Redistributable 下載](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)

