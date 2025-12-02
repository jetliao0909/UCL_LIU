# Rust 跨平台 GUI 框架比較

## 候選字視窗需求
- 輕量級（檔案大小小）
- 跨平台（Windows、Linux、macOS）
- 簡單 UI（候選字列表、狀態顯示）
- 無邊框視窗
- 透明背景（可選）
- 跟隨游標位置

## 選項比較

### 1. **fltk-rs** ⭐⭐⭐⭐⭐（最推薦）

**優點**：
- ✅ **最輕量**：二進制檔案約 1MB，無需額外 DLL
- ✅ **跨平台**：Windows、Linux、macOS 都支援
- ✅ **快速啟動**：啟動速度快
- ✅ **成熟穩定**：FLTK 是成熟的 C++ 庫，Rust 綁定穩定
- ✅ **簡單 API**：適合簡單 UI
- ✅ **無依賴**：靜態連結，單一執行檔

**缺點**：
- ❌ 外觀較舊（原生外觀不如其他框架現代）
- ❌ 文檔相對較少
- ❌ 複雜 UI 支援有限

**檔案大小**：~1MB（包含在最終 exe 中）

**適用場景**：輸入法候選字視窗（簡單 UI，需要輕量）

```toml
fltk = { version = "1.4", features = ["no-libpng", "no-libjpeg"] }
```

---

### 2. **egui** ⭐⭐⭐⭐

**優點**：
- ✅ **跨平台**：Windows、Linux、macOS、Web
- ✅ **Immediate Mode**：開發簡單，適合快速原型
- ✅ **現代化 UI**：外觀現代
- ✅ **活躍社區**：文檔豐富，更新頻繁
- ✅ **遊戲引擎整合**：可整合到遊戲中

**缺點**：
- ❌ **檔案較大**：約 2-3MB（包含在 exe 中）
- ❌ 需要 winit（視窗管理）
- ❌ Immediate Mode 可能不適合複雜 UI

**檔案大小**：~2-3MB

**適用場景**：需要現代化 UI，檔案大小不是主要考量

```toml
egui = "0.24"
winit = "0.29"
eframe = "0.24"  # 或直接使用 egui
```

---

### 3. **iced** ⭐⭐⭐

**優點**：
- ✅ **跨平台**：Windows、Linux、macOS
- ✅ **Elm 風格**：函數式編程，類型安全
- ✅ **現代化 UI**：外觀現代
- ✅ **文檔完善**：有完整的書和範例

**缺點**：
- ❌ **檔案較大**：約 3-5MB
- ❌ 學習曲線較陡（需要理解 Elm 架構）
- ❌ 對於簡單 UI 可能過於複雜

**檔案大小**：~3-5MB

**適用場景**：需要複雜 UI，願意學習 Elm 架構

```toml
iced = "0.10"
```

---

### 4. **slint** ⭐⭐⭐⭐

**優點**：
- ✅ **跨平台**：Windows、Linux、macOS、Web、嵌入式
- ✅ **原生外觀**：支援各平台原生樣式（Fluent、Material、Cupertino）
- ✅ **輕量**：比 egui/iced 小
- ✅ **現代化**：較新的框架，設計現代
- ✅ **多語言支援**：支援 Rust、C++、JavaScript

**缺點**：
- ❌ 相對較新，生態系統還在發展
- ❌ 檔案大小中等（約 2-3MB）
- ❌ 需要學習 Slint 專屬語法（類似 QML）

**檔案大小**：~2-3MB

**適用場景**：需要原生外觀，願意學習新語法

```toml
slint = "1.4"
```

---

### 5. **winit + 自繪** ⭐⭐

**優點**：
- ✅ **最輕量**：完全控制，只包含需要的功能
- ✅ **跨平台**：winit 支援所有平台
- ✅ **完全控制**：可以實作任何 UI

**缺點**：
- ❌ **開發時間長**：需要自己實作所有 UI 組件
- ❌ 需要自己處理文字渲染、佈局等
- ❌ 維護成本高

**檔案大小**：~500KB（僅 winit）

**適用場景**：需要完全控制，有充足開發時間

```toml
winit = "0.29"
# 需要自己實作 UI 渲染
```

---

## 推薦方案

### 方案 1：fltk-rs（最推薦）⭐⭐⭐⭐⭐

**理由**：
- 最輕量（~1MB）
- 跨平台支援完整
- 適合簡單 UI（候選字視窗）
- 單一執行檔，無依賴

**實作範例**：
```rust
use fltk::{app::*, window::*, text::*};

struct CandidateWindow {
    window: Window,
    text: TextDisplay,
}

impl CandidateWindow {
    fn new() -> Self {
        let mut window = Window::new(100, 100, 300, 200, "候選字");
        window.set_border(false);
        window.make_resizable(false);
        
        let mut text = TextDisplay::new(10, 10, 280, 180, "");
        text.set_buffer(TextBuffer::default());
        
        window.end();
        window.show();
        
        Self { window, text }
    }
    
    fn update_candidates(&mut self, candidates: &[String]) {
        let mut buffer = TextBuffer::default();
        for (i, candidate) in candidates.iter().enumerate() {
            buffer.append(&format!("{}: {}\n", i, candidate));
        }
        self.text.set_buffer(buffer);
    }
}
```

---

### 方案 2：egui（如果需要現代化 UI）⭐⭐⭐⭐

**理由**：
- 現代化 UI
- 開發簡單
- 跨平台完整
- 檔案稍大但可接受（~2-3MB）

---

### 方案 3：混合方案（最佳平衡）⭐⭐⭐⭐⭐

**建議**：
- **主要使用 fltk-rs**：候選字視窗（簡單、輕量）
- **未來可選 egui**：如果需要更複雜的設定視窗

這樣可以：
- 保持檔案小（候選字視窗用 fltk）
- 未來擴展性（設定視窗用 egui）
- 最佳性能（簡單 UI 用輕量框架）

---

## 檔案大小比較（Release 版本）

| 框架 | 檔案大小 | 說明 |
|------|---------|------|
| fltk-rs | ~1MB | 最輕量 |
| winit + 自繪 | ~500KB | 需要自己實作 UI |
| slint | ~2-3MB | 原生外觀 |
| egui | ~2-3MB | Immediate Mode |
| iced | ~3-5MB | Elm 風格 |

---

## 最終建議

**對於輸入法候選字視窗，推薦使用 fltk-rs**：

1. ✅ **最輕量**：檔案最小，符合輸入法需求
2. ✅ **跨平台**：Windows、Linux、macOS 都支援
3. ✅ **簡單 UI**：候選字列表不需要複雜 UI
4. ✅ **成熟穩定**：FLTK 是成熟的庫
5. ✅ **無依賴**：單一執行檔

如果未來需要更複雜的 UI（如設定視窗），可以考慮：
- 使用 egui 作為補充
- 或繼續使用 fltk-rs（功能足夠）

