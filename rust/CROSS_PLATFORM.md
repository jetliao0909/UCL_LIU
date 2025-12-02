# 跨平台支援計劃

## 可行性分析

支援 macOS 確實不難！核心的輸入法邏輯（`dictionary.rs`、`input_method.rs`）已經是純 Rust 實現，完全跨平台。只需要將平台特定的部分（鍵盤鉤子、輸入模擬）進行條件編譯即可。

## 當前架構分析

### ✅ 已跨平台的部分（可直接使用）
- `dictionary.rs` - 字碼表載入和查詢（純 Rust）
- `input_method.rs` - 輸入法邏輯處理（純 Rust）
- `config.rs` - 配置檔案處理（純 Rust）
- JSON 處理、錯誤處理、日誌等

### 🔧 需要平台特定的部分

1. **鍵盤鉤子** (`keyboard_hook.rs`)
   - Windows: `SetWindowsHookExW` (windows crate)
   - macOS: `CGEventTap` (core-graphics crate)
   - Linux: X11 使用 `XGrabKey` / `XSelectInput` (x11 crate) 或 `xcb` (xcb crate)
   - Linux Wayland: 需要 compositor 特定 API（較複雜）

2. **輸入模擬** (`input_simulator.rs`)
   - **方案 A：原生 API**（當前實現）
     - Windows: `SendInput` (windows crate)
     - macOS: `CGEvent` (core-graphics crate)
     - Linux X11: `XTestFakeKeyEvent` (XTest 擴展) 或 `uinput` (需要 root)
     - Linux Wayland: 需要 compositor 特定 API（較複雜）
   - **方案 B：使用 Enigo**（推薦用於跨平台）
     - 統一 API，自動處理平台差異
     - 支援 Windows、macOS、Linux X11
     - Wayland 有實驗性支援
   - 剪貼簿操作：`arboard` crate 已經跨平台 ✅

3. **系統托盤** (`tray.rs`)
   - `tray-icon` crate 已經支援跨平台 ✅

## 實現方案

### 方案 1：條件編譯 + 原生 API（當前實現，性能最佳）

使用 Rust 的條件編譯功能，創建平台特定的模組，直接使用各平台的原生 API。

使用 Rust 的條件編譯功能，創建平台特定的模組：

```
src/
├── keyboard_hook/
│   ├── mod.rs
│   ├── windows.rs      #[cfg(target_os = "windows")]
│   ├── macos.rs         #[cfg(target_os = "macos")]
│   └── linux.rs         #[cfg(target_os = "linux")]
├── input_simulator/
│   ├── mod.rs
│   ├── windows.rs       #[cfg(target_os = "windows")]
│   ├── macos.rs         #[cfg(target_os = "macos")]
│   └── linux.rs         #[cfg(target_os = "linux")]
└── ...
```

**優點：**
- 性能最佳，直接使用平台 API
- 功能最完整，可充分利用平台特性
- 對平台特定功能的控制最強

**缺點：**
- 需要為每個平台單獨實現
- 代碼量較多，維護成本較高

### 方案 2：使用 Enigo 統一 API（推薦用於快速跨平台）

使用 Enigo 作為統一的輸入模擬接口，簡化跨平台實現：

```
src/
├── input_simulator/
│   ├── mod.rs           # 統一的 trait 接口
│   └── enigo_impl.rs    # 使用 Enigo 的實現（所有平台共用）
└── ...
```

**實現範例：**
```rust
// src/input_simulator/mod.rs
use enigo::*;

pub struct InputSimulator {
    enigo: Enigo,
}

impl InputSimulator {
    pub fn new() -> Result<Self> {
        Ok(Self {
            enigo: Enigo::new(),
        })
    }
    
    /// 發送文字（跨平台）
    pub fn send_text(&mut self, text: &str) -> Result<()> {
        self.enigo.text(text);
        Ok(())
    }
    
    /// 發送按鍵組合（例如 Ctrl+V）
    pub fn send_key_combination(&mut self, keys: &[Key]) -> Result<()> {
        // Enigo 自動處理平台差異
        for key in keys {
            self.enigo.key(*key, Direction::Press);
        }
        for key in keys.iter().rev() {
            self.enigo.key(*key, Direction::Release);
        }
        Ok(())
    }
}
```

**優點：**
- 代碼簡潔，單一實現適用所有平台
- 維護成本低，無需處理平台差異
- 快速實現跨平台支援

**缺點：**
- 功能可能不如原生 API 豐富
- 對平台特定功能的控制較少
- Wayland 支援不完整

### 方案 3：混合方案（推薦）

結合兩種方案的優點：
- **鍵盤鉤子**：使用條件編譯 + 原生 API（需要低層級控制）
- **輸入模擬**：使用 Enigo（簡化實現，統一 API）

這樣可以在保持鍵盤鉤子靈活性的同時，簡化輸入模擬的跨平台實現。

### 方案 4：其他跨平台 crate

使用現有的跨平台 crate：

#### Enigo - 跨平台輸入模擬（推薦用於簡化實現）

**Enigo** 是一個成熟的跨平台輸入模擬庫，支援：
- ✅ **Windows** - 完全支援
- ✅ **macOS** - 完全支援（需要輔助功能權限）
- ✅ **Linux X11** - 完全支援
- ⚠️ **Linux Wayland** - 實驗性支援（可能有問題）
- ⚠️ **Linux libei** - 實驗性支援

**優點：**
- 統一的 API，無需處理平台差異
- 自動處理不同平台的實現細節
- 代碼更簡潔，維護更容易
- 適合快速實現跨平台支援

**缺點：**
- 功能可能不如原生 API 豐富
- 對平台特定功能的控制較少
- Wayland 支援不完整

**使用範例：**
```rust
use enigo::*;

let mut enigo = Enigo::new();

// 發送文字（自動處理平台差異）
enigo.text("Hello, World!");

// 發送按鍵組合
enigo.key(Key::Control, Direction::Press);
enigo.key(Key::Unicode('c'), Direction::Click);
enigo.key(Key::Control, Direction::Release);
```

**其他跨平台 crate：**
- `rdev` - 跨平台鍵盤/滑鼠事件監聽（支援 Linux、macOS、Windows）
- `global-hotkey` - 跨平台全域熱鍵
- `x11rb` / `xcb` - Linux X11 API 綁定

## macOS 實現要點

### 1. 鍵盤鉤子 (CGEventTap)

```rust
// macOS 使用 CGEventTap
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, EventRef,
};

// 創建事件監聽
let event_tap = CGEventTap::new(
    CGEventTapLocation::HID,
    CGEventTapPlacement::HeadInsertEventTap,
    CGEventTapOptions::Default,
    CGEventType::keyDown | CGEventType::keyUp,
    callback,
)?;
```

### 2. 輸入模擬

#### 方案 A：使用 Enigo（推薦）

```rust
// 使用 Enigo，自動處理平台差異
use enigo::*;

let mut enigo = Enigo::new();

// 發送文字
enigo.text("Hello, World!");

// 發送按鍵組合（Ctrl+V）
enigo.key(Key::Control, Direction::Press);
enigo.key(Key::Unicode('v'), Direction::Click);
enigo.key(Key::Control, Direction::Release);
```

#### 方案 B：使用原生 API (CGEvent)

```rust
// macOS 使用 CGEvent
use core_graphics::event::{CGEvent, CGEventFlags, CGKeyCode};

// 發送鍵盤事件
let event = CGEvent::new_keyboard_event(key_code, true)?;
event.post(CGEventTapLocation::HID);
```

### 3. 權限要求

macOS 需要用戶授予：
- **輔助功能權限**（Accessibility）：用於鍵盤鉤子和輸入模擬
- 需要在 `Info.plist` 中聲明權限

## 需要的依賴

### 方案 A：使用 Enigo（推薦用於快速跨平台）

```toml
[dependencies]
enigo = "0.1"  # 跨平台輸入模擬（支援 Windows、macOS、Linux X11）
```

**優點：** 單一依賴，自動處理所有平台

### 方案 B：原生 API（當前實現，性能最佳）

#### macOS 依賴

```toml
[target.'cfg(target_os = "macos")'.dependencies]
core-graphics = "0.23"  # macOS 圖形和事件 API
cocoa = "0.25"          # macOS 系統 API（如果需要）
```

#### Linux 依賴

```toml
[target.'cfg(target_os = "linux")'.dependencies]
# X11 支援（較常見）
x11rb = "0.12"          # X11 API 綁定（推薦，更現代）
# 或
xcb = "1.3"             # X11 API 綁定（傳統方式）

# Wayland 支援（較新，但實現複雜）
# wayland-client = "0.31"  # Wayland 客戶端 API

# 輸入模擬
# uinput-sys = "0.1"    # Linux uinput（需要 root，不推薦）
```

#### Windows 依賴（已存在）

```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.52", features = [...] }
```

### 方案 C：混合方案（推薦）

```toml
# 所有平台共用
[dependencies]
enigo = "0.1"  # 用於輸入模擬

# Windows 特定（鍵盤鉤子）
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.52", features = [...] }

# macOS 特定（鍵盤鉤子）
[target.'cfg(target_os = "macos")'.dependencies]
core-graphics = "0.23"

# Linux 特定（鍵盤鉤子）
[target.'cfg(target_os = "linux")'.dependencies]
x11rb = "0.12"
```

## Linux 實現要點

### 1. 鍵盤鉤子

#### X11 方式（推薦，較簡單）

```rust
// Linux X11 使用 XGrabKey 或 XSelectInput
use x11rb::connection::Connection;
use x11rb::xcb_ffi::XCBConnection;

// 監聽鍵盤事件
let conn = XCBConnection::connect(None)?;
let root = conn.setup().roots[0].root;
conn.grab_key(...)?;
```

#### Wayland 方式（較複雜）

Wayland 沒有全域鍵盤鉤子的標準 API，需要：
- 使用 compositor 特定 API（如 GNOME、KDE）
- 或使用 `libinput`（需要特殊權限）
- 或實現為 Wayland 輸入法（IME）

### 2. 輸入模擬

#### 方案 A：使用 Enigo（推薦）

```rust
// 使用 Enigo，自動處理 X11/Wayland
use enigo::*;

let mut enigo = Enigo::new();
enigo.text("Hello, World!");
```

**注意：** Enigo 在 Linux 上主要透過 X11 工作，Wayland 有實驗性支援但可能有問題。

#### 方案 B：使用原生 API (X11)

```rust
// Linux X11 使用 XTest 擴展
use x11rb::protocol::xtest::XTestFakeKeyEvent;

// 發送鍵盤事件
conn.xtest_fake_key_event(...)?;
```

#### uinput 方式（需要 root，不推薦）

```rust
// Linux uinput（需要 root 權限）
// 不推薦，因為需要特殊權限
```

### 3. X11 vs Wayland 檢測

```rust
// 檢測顯示伺服器類型
let display_server = std::env::var("WAYLAND_DISPLAY")
    .map(|_| "wayland")
    .or_else(|_| std::env::var("DISPLAY").map(|_| "x11"))
    .unwrap_or("unknown");
```

### 4. 權限要求

Linux 通常需要：
- **X11**：一般不需要特殊權限（如果使用 XTest）
- **Wayland**：可能需要特殊權限或實現為 IME
- **uinput**：需要 root 權限（不推薦）

## 實現步驟

### 方案 A：使用 Enigo（推薦用於快速跨平台）

1. **添加 Enigo 依賴**
   ```toml
   [dependencies]
   enigo = "0.1"
   ```

2. **重構輸入模擬器**
   - 修改 `input_simulator.rs` 使用 Enigo
   - 統一 API，無需平台特定代碼

3. **實現鍵盤鉤子（仍需平台特定）**
   - 將 `keyboard_hook.rs` 拆分為平台特定模組
   - Windows: 使用現有實現
   - macOS: 實現 CGEventTap
   - Linux: 實現 X11 鍵盤鉤子

4. **測試和調試**
   - 在各平台環境中測試
   - 確保功能一致性

### 方案 B：完全使用原生 API（性能最佳）

1. **重構模組結構**
   - 將 `keyboard_hook.rs` 拆分為 `keyboard_hook/windows.rs`、`keyboard_hook/macos.rs`、`keyboard_hook/linux.rs`
   - 將 `input_simulator.rs` 拆分為 `input_simulator/windows.rs`、`input_simulator/macos.rs`、`input_simulator/linux.rs`
   - 創建統一的 trait 接口

2. **實現 macOS 版本**
   - 實現 macOS 鍵盤鉤子
   - 實現 macOS 輸入模擬（CGEvent）
   - 處理 macOS 權限

3. **實現 Linux 版本**
   - 優先實現 X11 支援（較簡單）
   - 實現 X11 鍵盤鉤子
   - 實現 X11 輸入模擬（XTest）
   - Wayland 支援可以後續添加（較複雜）

4. **測試和調試**
   - 在 macOS 環境中測試
   - 在 Linux X11 環境中測試
   - 確保功能一致性

### 方案 C：混合方案（推薦）

1. **鍵盤鉤子**：使用方案 B（條件編譯 + 原生 API）
2. **輸入模擬**：使用方案 A（Enigo）
3. 結合兩種方案的優點，在保持靈活性的同時簡化實現

## 預估工作量

### 方案 A：使用 Enigo

#### macOS
- **鍵盤鉤子實現**：2-3 天（仍需平台特定）
- **輸入模擬實現**：0.5 天（使用 Enigo，幾乎無需額外工作）
- **權限處理和測試**：1-2 天
- **總計**：約 4-5 天

#### Linux (X11)
- **鍵盤鉤子實現**：2-3 天（仍需平台特定）
- **輸入模擬實現**：0.5 天（使用 Enigo，幾乎無需額外工作）
- **X11/Wayland 檢測**：0.5 天
- **測試**：1-2 天
- **總計**：約 4-5 天

**優點：** 輸入模擬部分工作量大幅減少

### 方案 B：完全使用原生 API

#### macOS
- **鍵盤鉤子實現**：2-3 天
- **輸入模擬實現**：1-2 天
- **權限處理和測試**：1-2 天
- **總計**：約 1 週

#### Linux (X11)
- **鍵盤鉤子實現**：2-3 天
- **輸入模擬實現**：1-2 天
- **X11/Wayland 檢測**：0.5 天
- **測試**：1-2 天
- **總計**：約 1 週

### Linux (Wayland)
- **Wayland 支援**：較複雜，可能需要 1-2 週
- **建議**：先實現 X11 支援，Wayland 可以後續添加
- **注意**：Enigo 對 Wayland 的支援是實驗性的，可能不穩定

## 優勢

1. **核心邏輯共用**：輸入法邏輯完全不需要修改
2. **Rust 條件編譯**：編譯時自動選擇正確的平台實現
3. **單一程式碼庫**：維護更方便
4. **測試覆蓋**：核心邏輯的測試可以共用

## Linux 特殊考量

### X11 vs Wayland

Linux 有兩個主要的顯示伺服器：
1. **X11**（傳統，較簡單）
   - 有標準的 API（X11、xcb）
   - 鍵盤鉤子和輸入模擬相對簡單
   - 推薦優先實現

2. **Wayland**（現代，較複雜）
   - 沒有全域鍵盤鉤子的標準 API
   - 需要 compositor 特定實現
   - 或實現為 Wayland IME（輸入法框架）
   - 可以後續添加

### 建議實現順序

1. **Windows** ✅（已完成）
2. **macOS**（相對簡單）
3. **Linux X11**（較簡單，推薦優先）
4. **Linux Wayland**（較複雜，可以後續）

## 結論

支援 macOS 和 Linux 確實不難！主要工作集中在平台特定的 API 封裝上，核心的輸入法邏輯已經完全跨平台了。

### 推薦方案

**混合方案（方案 C）**：
- **鍵盤鉤子**：使用條件編譯 + 原生 API（需要低層級控制）
- **輸入模擬**：使用 Enigo（簡化實現，統一 API）

這樣可以在保持鍵盤鉤子靈活性的同時，大幅簡化輸入模擬的跨平台實現。

### Linux 特別說明

- **X11 支援**：相對簡單，類似 macOS 的複雜度
- **Wayland 支援**：較複雜，建議先實現 X11，Wayland 可以後續添加
- **Enigo 在 Linux**：主要透過 X11 工作，Wayland 有實驗性支援但可能不穩定
- 使用 Rust 的條件編譯功能，可以優雅地實現跨平台支援

### 實現順序建議

1. **Windows** ✅（已完成）
2. **macOS**（使用 Enigo 簡化輸入模擬）
3. **Linux X11**（使用 Enigo 簡化輸入模擬）
4. **Linux Wayland**（較複雜，可以後續添加）

### Enigo 使用建議

- ✅ **適合**：輸入模擬（發送文字、按鍵組合）
- ✅ **優點**：代碼簡潔、維護容易、快速實現
- ⚠️ **限制**：Wayland 支援不完整、功能可能不如原生 API 豐富
- 💡 **建議**：對於輸入模擬，Enigo 是很好的選擇；對於鍵盤鉤子，仍建議使用原生 API

