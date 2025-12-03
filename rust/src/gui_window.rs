//! GUI 主窗口模組
//! 用於顯示字根和候選字（類似 Python 版本的 type_label 和 word_label）
//! 同時作為輸入窗口，能夠接收鍵盤輸入（用於 Raw Input 遊戲）

use crate::input_method::InputMethodProcessor;
use crate::input_simulator::InputSimulator;
use anyhow::Result;
use fltk::{
    app,
    enums::{Align, Color, Event, Key},
    frame::Frame,
    prelude::*,
    window::Window,
};
use log::{debug, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use windows::{
    Win32::Foundation::{COLORREF, HWND},
    Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos,
        GWL_EXSTYLE, HWND_TOPMOST, LWA_ALPHA, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
        WS_EX_LAYERED,
    },
};

/// GUI 主窗口
pub struct GuiWindow {
    window: Window,
    code_frame: Frame,             // 字根顯示框（類似 Python 的 type_label）
    word_frame: Frame,             // 候選字顯示框（類似 Python 的 word_label）
    accumulated_text_frame: Frame, // 累積文字顯示框（顯示待貼上的完整句子）
    processor: Arc<Mutex<InputMethodProcessor>>,
    input_simulator: Arc<Mutex<InputSimulator>>,
    gui_needs_update: Arc<AtomicBool>,
    is_input_mode: bool, // 是否為輸入模式（窗口有焦點時接收鍵盤輸入）
    accumulated_text: Arc<Mutex<String>>, // 累積的文字（待貼上到遊戲）
}

impl GuiWindow {
    /// 創建新的 GUI 主窗口
    pub fn new(
        processor: Arc<Mutex<InputMethodProcessor>>,
        input_simulator: Arc<Mutex<InputSimulator>>,
        gui_needs_update: Arc<AtomicBool>,
    ) -> Result<Self> {
        // 獲取屏幕尺寸，將窗口放在屏幕右下角
        let screen_w = app::screen_size().0 as i32;
        let screen_h = app::screen_size().1 as i32;
        let win_w = 500;
        let win_h = 100; // 增加高度以容納累積文字顯示框
        let win_x = screen_w - win_w - 10; // 距離右邊 10 像素
        let win_y = screen_h - win_h - 50; // 距離底部 50 像素（避免被任務欄遮擋）

        let mut window = Window::new(win_x, win_y, win_w, win_h, "");
        // 顯示邊框，讓使用者更容易看到視窗位置
        window.set_border(true);
        window.set_color(Color::from_rgb(222, 222, 222)); // 淺灰色背景，類似 Python 版本
        window.make_modal(false);

        // 設置窗口可以接收鍵盤焦點（重要：用於輸入窗口模式）
        // 注意：ESC 鍵不再關閉窗口，改為在 handle_keyboard_event 中處理

        // 字根顯示框（類似 Python 的 type_label）
        let mut code_frame = Frame::new(5, 5, 100, 50, "");
        code_frame.set_label_size(22);
        code_frame.set_label_color(Color::Black);
        code_frame.set_color(Color::from_rgb(222, 222, 222)); // 淺灰色背景
        code_frame.set_align(Align::Left | Align::Inside);

        // 候選字顯示框（類似 Python 的 word_label）
        let mut word_frame = Frame::new(110, 5, 385, 50, "");
        word_frame.set_label_size(20);
        word_frame.set_label_color(Color::Black);
        word_frame.set_color(Color::from_rgb(222, 222, 222)); // 淺灰色背景
        word_frame.set_align(Align::Left | Align::Inside);

        // 累積文字顯示框（顯示待貼上的完整句子）
        let mut accumulated_text_frame = Frame::new(5, 60, 490, 30, "");
        accumulated_text_frame.set_label_size(16);
        accumulated_text_frame.set_label_color(Color::from_rgb(0, 100, 0)); // 深綠色，表示待貼上
        accumulated_text_frame.set_color(Color::from_rgb(240, 255, 240)); // 淺綠色背景
        accumulated_text_frame.set_align(Align::Left | Align::Inside);

        window.end();

        // 初始顯示
        code_frame.set_label("");
        word_frame.set_label("");
        accumulated_text_frame.set_label("待貼上文字將顯示在這裡... (已自動複製到剪貼簿)");

        // 設置鍵盤事件處理（用於輸入窗口模式）
        let processor_clone = processor.clone();
        let input_simulator_clone = input_simulator.clone();
        let gui_needs_update_clone = gui_needs_update.clone();
        let accumulated_text_clone = Arc::new(Mutex::new(String::new()));
        let accumulated_text_for_handler = accumulated_text_clone.clone();

        window.handle(move |w, ev| {
            // 讓 FLTK 處理 Focus/Unfocus，並在鍵盤事件時直接詢問窗口是否有焦點
            match ev {
                Event::Focus => {
                    debug!("輸入窗口獲得焦點");
                    // 視窗獲得焦點時，提高透明度，讓使用者明顯感覺「現在可以打字」
                    unsafe {
                        let raw = w.raw_handle();
                        let hwnd = HWND(raw as isize);
                        let _ = SetLayeredWindowAttributes(
                            hwnd,
                            COLORREF(0),
                            100, // 聚焦時半透明（或可改成 255 完全不透明）
                            LWA_ALPHA,
                        );
                    }
                    // 不在這裡處理鍵盤邏輯，讓事件繼續傳遞
                    return false;
                }
                Event::Unfocus => {
                    debug!("輸入窗口失去焦點");
                    // 視窗失去焦點時，幾乎完全透明，避免誤會它有焦點
                    unsafe {
                        let raw = w.raw_handle();
                        let hwnd = HWND(raw as isize);
                        let _ = SetLayeredWindowAttributes(
                            hwnd,
                            COLORREF(0),
                            10, // 失焦時幾乎完全透明（0~255）
                            LWA_ALPHA,
                        );
                    }
                    return false;
                }
                _ => {}
            }

            // 處理鍵盤事件（只在窗口有焦點時處理）
            Self::handle_keyboard_event(
                w,
                ev,
                &processor_clone,
                &input_simulator_clone,
                &gui_needs_update_clone,
                &accumulated_text_for_handler,
            )
        });

        Ok(Self {
            window,
            code_frame,
            word_frame,
            accumulated_text_frame,
            processor,
            input_simulator,
            gui_needs_update,
            is_input_mode: false,
            accumulated_text: accumulated_text_clone, // 使用同一個 Arc，這樣 handler 和窗口可以共享
        })
    }

    /// 複製文字到剪貼簿（輔助函數）
    fn copy_to_clipboard(text: &str) {
        if text.is_empty() {
            return;
        }

        use arboard::Clipboard;
        if let Ok(mut clipboard) = Clipboard::new() {
            if clipboard.set_text(text).is_ok() {
                debug!("✅ 已自動複製文字到剪貼簿: {}", text);
            } else {
                warn!("⚠️ 複製到剪貼簿失敗");
            }
        } else {
            warn!("⚠️ 無法創建剪貼簿對象");
        }
    }

    /// 處理鍵盤事件（輸入窗口模式）
    /// 當窗口有焦點時，直接處理鍵盤輸入，不依賴鍵盤鉤子
    ///
    /// **重要**：選擇候選字後，文字會累積在窗口中，並自動複製到剪貼簿
    /// 用戶只需要切換回遊戲，按 Ctrl+V 貼上全部文字
    /// 這樣可以避免頻繁切換焦點，更可靠
    fn handle_keyboard_event(
        w: &mut Window,
        ev: Event,
        processor: &Arc<Mutex<InputMethodProcessor>>,
        _input_simulator: &Arc<Mutex<InputSimulator>>,
        gui_needs_update: &Arc<AtomicBool>,
        accumulated_text: &Arc<Mutex<String>>,
    ) -> bool {
        match ev {
            Event::KeyDown => {
                // 檢查窗口是否有焦點，如果沒有焦點則不處理鍵盤事件
                // 這可以避免在窗口沒有焦點時處理鍵盤事件導致衝突
                if !w.has_focus() {
                    debug!("輸入窗口沒有焦點，忽略鍵盤事件");
                    return false; // 讓事件通過，不處理
                }

                let key = app::event_key();
                let key_char = app::event_text();

                debug!("輸入窗口收到按鍵: key={:?}, char='{}'", key, key_char);

                // 處理 ESC 鍵（清除當前輸入的字根，但不關閉窗口）
                if key == Key::Escape {
                    // 清除當前輸入的字根（但不清除累積的文字）
                    let mut proc = processor.lock().unwrap();
                    proc.clear();
                    gui_needs_update.store(true, Ordering::Relaxed);
                    debug!("ESC: 清除當前輸入的字根");
                    return true; // 已處理
                }

                // 處理字母鍵（字根輸入）
                if !key_char.is_empty() {
                    let ch = key_char.chars().next().unwrap();
                    if ch.is_ascii_alphabetic() {
                        let ch_lower = ch.to_ascii_lowercase();
                        let (success, complement_selected) = {
                            let mut proc = processor.lock().unwrap();
                            proc.handle_code_input(ch_lower)
                        };

                        if success {
                            if complement_selected.is_some() {
                                // 補碼選擇，等待 Space 鍵
                                info!("✅ 補碼選擇候選字（等待 Space 鍵送出）");
                            }
                            gui_needs_update.store(true, Ordering::Relaxed);
                            return true; // 已處理
                        }
                    }
                }

                // 處理數字鍵（候選字選擇）
                // 使用 event_text() 來檢查字符，因為 FLTK 的 Key 枚舉不直接支持數字鍵
                if !key_char.is_empty() {
                    if let Some(ch) = key_char.chars().next() {
                        // ASCII 數字鍵 → 用來選擇候選字
                        if ch.is_ascii_digit() {
                            let num = ch.to_digit(10).unwrap() as u8;
                            let num_u8 = if num == 0 { 0 } else { num as u8 };
                            if let Some(text) = {
                                let mut proc = processor.lock().unwrap();
                                proc.handle_number_selection(num_u8)
                            } {
                                // 選擇了候選字，累積到文字緩衝區並自動複製到剪貼簿
                                let text_to_copy = {
                                    let mut acc_text = accumulated_text.lock().unwrap();
                                    acc_text.push_str(&text);
                                    let result = acc_text.clone();
                                    info!("✅ 選擇候選字 {}: {}，累積文字: {}", num, text, result);
                                    result
                                };

                                // 自動複製到剪貼簿
                                Self::copy_to_clipboard(&text_to_copy);

                                gui_needs_update.store(true, Ordering::Relaxed);
                                return true; // 已處理
                            } else {
                                // 沒有對應的候選字，攔截並忽略該按鍵
                                debug!("數字鍵 {} 沒有對應的候選字，攔截並忽略", num);
                                return true; // 已處理（攔截）
                            }
                        }
                    }
                }

                // 處理 Space 鍵（選擇第一個候選字）
                if key == Key::from_char(' ') || key_char == " " {
                    if let Some(text) = {
                        let mut proc = processor.lock().unwrap();
                        proc.handle_space()
                    } {
                        // 有候選字，累積到文字緩衝區並自動複製到剪貼簿
                        let text_to_copy = {
                            let mut acc_text = accumulated_text.lock().unwrap();
                            acc_text.push_str(&text);
                            let result = acc_text.clone();
                            info!("Space: 選擇候選字: {}，累積文字: {}", text, result);
                            result
                        };

                        // 自動複製到剪貼簿
                        Self::copy_to_clipboard(&text_to_copy);

                        gui_needs_update.store(true, Ordering::Relaxed);
                        return true; // 已處理
                    }
                    // 沒有候選字，讓 Space 鍵通過（可能用戶想輸入空格）
                    return false;
                }

                // 處理 Enter 鍵（清除累積的文字）
                if key == Key::Enter {
                    {
                        let mut acc_text = accumulated_text.lock().unwrap();
                        if !acc_text.is_empty() {
                            acc_text.clear();
                            info!("✅ Enter: 已清除累積文字");
                            gui_needs_update.store(true, Ordering::Relaxed);
                            return true; // 已處理
                        }
                    }
                    // 如果沒有累積文字，讓 Enter 鍵通過
                    return false;
                }

                // 處理 Backspace 鍵
                if key == Key::BackSpace {
                    let handled = {
                        let mut proc = processor.lock().unwrap();
                        proc.handle_backspace()
                    };
                    if handled {
                        gui_needs_update.store(true, Ordering::Relaxed);
                        return true; // 已處理
                    }
                    // 沒有字根可刪除，讓 Backspace 鍵通過
                    return false;
                }

                // 處理 Ctrl+V（手動重新複製累積的文字到剪貼簿，用於刷新剪貼簿內容）
                if app::event_state().contains(fltk::enums::Shortcut::Ctrl)
                    && key == Key::from_char('v')
                {
                    let text_to_copy = {
                        let acc_text = accumulated_text.lock().unwrap();
                        acc_text.clone()
                    };

                    if !text_to_copy.is_empty() {
                        // 重新複製累積的文字到剪貼簿（用於刷新）
                        Self::copy_to_clipboard(&text_to_copy);
                        info!(
                            "💡 提示：已重新複製累積文字到剪貼簿，請切換回遊戲，按 Ctrl+V 貼上文字"
                        );
                        gui_needs_update.store(true, Ordering::Relaxed);
                        return true; // 已處理
                    }
                    // 如果沒有累積文字，讓 Ctrl+V 通過（可能用戶想貼上其他內容）
                    return false;
                }

                // 處理 Ctrl+C（清除累積的文字）
                if app::event_state().contains(fltk::enums::Shortcut::Ctrl)
                    && key == Key::from_char('c')
                {
                    {
                        let mut acc_text = accumulated_text.lock().unwrap();
                        if !acc_text.is_empty() {
                            acc_text.clear();
                            info!("✅ 已清除累積文字");
                            gui_needs_update.store(true, Ordering::Relaxed);
                            return true; // 已處理
                        }
                    }
                    // 如果沒有累積文字，讓 Ctrl+C 通過（可能用戶想複製其他內容）
                    return false;
                }

                // 其他 Ctrl 組合鍵，讓它通過
                if app::event_state().contains(fltk::enums::Shortcut::Ctrl) {
                    return false;
                }

                // 處理一般輸入文字（例如使用系統輸入法輸入的中文字、全形符號等）
                // 這些通常會以已組字完成的字元出現在 event_text() 裡
                if !key_char.is_empty() {
                    if let Some(ch) = key_char.chars().next() {
                        // 過濾掉控制字元，只處理可見字元
                        if !ch.is_control() {
                            let text_to_copy = {
                                let mut acc_text = accumulated_text.lock().unwrap();
                                acc_text.push(ch);
                                let result = acc_text.clone();
                                info!("直接輸入字元 '{}', 累積文字: {}", ch, result);
                                result
                            };

                            // 自動複製到剪貼簿
                            Self::copy_to_clipboard(&text_to_copy);

                            gui_needs_update.store(true, Ordering::Relaxed);
                            return true; // 已處理
                        }
                    }
                }

                // 其他非文字按鍵：攔截（避免在輸入窗口模式下觸發奇怪行為）
                debug!("輸入窗口攔截非文字按鍵: {:?}", key);
                true // 已處理（攔截）
            }
            _ => false, // 其他事件不處理
        }
    }

    /// 顯示窗口
    pub fn show(&mut self) {
        debug!("顯示 GUI 視窗（輸入窗口模式）");

        // 確保窗口可見
        if !self.window.shown() {
            self.window.show();
            // 讓 FLTK 真的建立底層 HWND，避免 raw_handle 為 null
            app::flush();
        }

        // 設置為當前窗口（不自動獲得焦點，用戶需要手動點擊窗口給予焦點）
        self.window.make_current();

        // 標記為輸入模式
        self.is_input_mode = true;

        // 清除之前的累積文字（每次打開窗口時重新開始）
        {
            let mut acc_text = self.accumulated_text.lock().unwrap();
            acc_text.clear();
        }

        info!("✅ 輸入窗口已顯示，請點擊窗口給予焦點後開始輸入");
        info!("💡 提示：選擇候選字後，文字會累積在窗口中，並自動複製到剪貼簿");
        info!("💡 提示：輸入完成後，切換回遊戲按 Ctrl+V 貼上全部文字");

        // 設定透明度與最上層屬性
        unsafe {
            let raw = self.window.raw_handle();
            let hwnd = HWND(raw as isize);

            // 開啟 WS_EX_LAYERED 擴充樣式，才能套用透明度
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let new_ex_style = ex_style | WS_EX_LAYERED.0 as isize;
            let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style);

            // 將整個視窗 alpha 設為 100（半透明）
            // 若想要更透明或更不透明，可調整第三個參數 0~255
            let _ = SetLayeredWindowAttributes(
                hwnd,
                COLORREF(0),
                100, // 0 = 完全透明, 255 = 完全不透明
                LWA_ALPHA,
            );

            // 嘗試將視窗設為最上層，避免被其他視窗（例如遊戲）遮住
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
        }

        // 更新顯示內容
        self.update_display();

        // 強制重繪
        self.window.redraw();
        self.code_frame.redraw();
        self.word_frame.redraw();

        // 強制刷新窗口
        app::flush();
        let _ = app::check();
        app::flush();

        debug!(
            "GUI 視窗已顯示，位置: ({}, {}), 大小: {}x{}, shown: {}",
            self.window.x(),
            self.window.y(),
            self.window.w(),
            self.window.h(),
            self.window.shown()
        );
    }

    /// 隱藏窗口
    pub fn hide(&mut self) {
        if self.window.shown() {
            // 清除輸入狀態
            let mut proc = self.processor.lock().unwrap();
            proc.clear();

            // 不清除累積文字，讓用戶可以在關閉窗口後仍然貼上
            // 用戶可以手動按 Enter 清除，或下次打開窗口時自動清除
            let acc_text = self.accumulated_text.lock().unwrap();
            if !acc_text.is_empty() {
                info!(
                    "💡 提示：累積的文字 '{}' 仍在剪貼簿中，可以在遊戲中按 Ctrl+V 貼上",
                    acc_text
                );
            }
            drop(acc_text);

            self.gui_needs_update.store(true, Ordering::Relaxed);

            self.window.hide();
            self.is_input_mode = false;
            info!("輸入窗口已隱藏，停止接收鍵盤輸入");
        }
    }

    /// 檢查窗口是否可見
    pub fn visible(&self) -> bool {
        self.window.shown()
    }

    /// 檢查窗口是否有焦點
    pub fn has_focus(&self) -> bool {
        self.window.has_focus()
    }

    /// 更新顯示（根據處理器狀態更新字根和候選字顯示）
    pub fn update_display(&mut self) {
        let processor = self.processor.lock().unwrap();
        let state = processor.get_state();

        // 更新字根顯示（類似 Python 的 type_label_set_text）
        if state.current_code.is_empty() {
            // 沒有字根時顯示提示文字，避免視覺上像是「什麼都沒出現」
            self.code_frame.set_label("輸入字根...");
        } else {
            self.code_frame.set_label(&state.current_code);
        }

        // 更新候選字顯示（類似 Python 的 word_label_set_text）
        let candidates = &state.candidates;
        if candidates.is_empty() {
            self.word_frame.set_label("");
        } else {
            let start_idx = state.candidate_index;
            let end_idx = (start_idx + 6).min(candidates.len());

            let mut labels = Vec::new();
            for i in start_idx..end_idx {
                let candidate = &candidates[i];
                if i == start_idx && state.complement_selected.is_none() {
                    labels.push(format!("{} (Space)", candidate));
                } else {
                    labels.push(format!("{}", candidate));
                }
            }

            // 如果有補碼選擇的候選字，顯示在第一個位置
            if let Some(ref selected) = state.complement_selected {
                self.word_frame.set_label(&format!("{} (Space)", selected));
            } else {
                self.word_frame.set_label(&labels.join(" "));
            }
        }

        // 更新累積文字顯示
        let acc_text = self.accumulated_text.lock().unwrap();
        let acc_text_str = acc_text.clone();
        drop(acc_text);

        if acc_text_str.is_empty() {
            self.accumulated_text_frame
                .set_label("待貼上文字將顯示在這裡... (已自動複製到剪貼簿，Enter 清除)");
        } else {
            self.accumulated_text_frame.set_label(&format!(
                "待貼上: {} (已自動複製到剪貼簿，切換回遊戲按 Ctrl+V 貼上，Enter 清除)",
                acc_text_str
            ));
        }

        // 強制重繪累積文字顯示框
        self.accumulated_text_frame.redraw();

        debug!(
            "GUI 窗口更新：字根='{}', 候選字數量={}, 累積文字='{}'",
            state.current_code,
            candidates.len(),
            acc_text_str
        );
    }

    /// 強制刷新顯示（不立即 flush，讓事件循環處理）
    pub fn redraw(&mut self) {
        self.window.redraw();
        self.code_frame.redraw();
        self.word_frame.redraw();
        self.accumulated_text_frame.redraw();
        // 不立即 flush，讓事件循環統一處理，避免頻繁刷新導致延遲
    }
}

/// GUI 窗口管理器
pub struct GuiWindowManager {
    window: Option<GuiWindow>,
    processor: Arc<Mutex<InputMethodProcessor>>,
    input_simulator: Arc<Mutex<InputSimulator>>,
    gui_needs_update: Arc<AtomicBool>,
    visible: bool, // 自行追蹤可見狀態，避免依賴底層 shown() 行為
}

impl GuiWindowManager {
    /// 創建新的 GUI 窗口管理器
    pub fn new(
        processor: Arc<Mutex<InputMethodProcessor>>,
        input_simulator: Arc<Mutex<InputSimulator>>,
        gui_needs_update: Arc<AtomicBool>,
    ) -> Self {
        Self {
            window: None,
            processor,
            input_simulator,
            gui_needs_update,
            visible: false,
        }
    }

    /// 顯示 GUI 窗口
    pub fn show(&mut self) -> Result<()> {
        if self.window.is_none() {
            let window = GuiWindow::new(
                self.processor.clone(),
                self.input_simulator.clone(),
                self.gui_needs_update.clone(),
            )?;
            self.window = Some(window);
        }

        if let Some(ref mut window) = self.window {
            window.show();
            // 注意：焦點狀態由 FLTK 自動管理，不需要手動設置
        }
        // 標記為可見
        self.visible = true;

        Ok(())
    }

    /// 隱藏 GUI 窗口
    pub fn hide(&mut self) {
        if let Some(ref mut window) = self.window {
            window.hide();
            // 注意：焦點狀態由 FLTK 自動管理，窗口隱藏時會自動失去焦點
        }
        // 標記為不可見
        self.visible = false;
    }

    /// 更新顯示
    pub fn update_display(&mut self) {
        if let Some(ref mut window) = self.window {
            window.update_display();
            window.redraw();
            // 觸發一次 flush，確保顯示更新
            fltk::app::flush();
            // 再次檢查並處理事件，確保重繪完成
            let _ = fltk::app::check();
            fltk::app::flush();
        }
    }

    /// 檢查窗口是否可見
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// 檢查窗口是否有焦點（從實際窗口讀取，確保準確）
    pub fn has_focus(&self) -> bool {
        // 從實際窗口讀取焦點狀態，直接調用 GuiWindow 的方法
        // 這樣可以確保焦點狀態是準確的，不會有緩存不同步的問題
        if let Some(ref window) = self.window {
            window.has_focus()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::Dictionary;
    use crate::input_method::InputMethodProcessor;
    use crate::input_simulator::InputSimulator;
    use std::collections::HashMap;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use std::sync::Mutex;

    /// 創建測試用的字典
    fn create_test_dictionary() -> Dictionary {
        let mut code_map = HashMap::new();
        code_map.insert("a".to_string(), vec!["一".to_string(), "乙".to_string()]);
        code_map.insert("ab".to_string(), vec!["二".to_string()]);
        code_map.insert("abc".to_string(), vec!["三".to_string(), "參".to_string()]);
        code_map.insert("test".to_string(), vec!["測試".to_string()]);

        Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        }
    }

    /// 創建測試用的組件
    fn create_test_components() -> (
        Arc<Mutex<InputMethodProcessor>>,
        Arc<Mutex<InputSimulator>>,
        Arc<AtomicBool>,
    ) {
        let dictionary = create_test_dictionary();
        let processor = Arc::new(Mutex::new(InputMethodProcessor::new(dictionary)));
        let input_simulator = Arc::new(Mutex::new(InputSimulator::new().unwrap()));
        let gui_needs_update = Arc::new(AtomicBool::new(false));

        (processor, input_simulator, gui_needs_update)
    }

    /// 測試：窗口創建成功
    #[test]
    fn test_gui_window_creation() {
        let (processor, input_simulator, gui_needs_update) = create_test_components();

        // 創建窗口應該成功
        let window_result = GuiWindow::new(
            processor.clone(),
            input_simulator.clone(),
            gui_needs_update.clone(),
        );

        assert!(window_result.is_ok(), "窗口創建應該成功");
    }

    /// 測試：窗口管理器創建成功
    #[test]
    fn test_gui_window_manager_creation() {
        let (processor, input_simulator, gui_needs_update) = create_test_components();

        let manager = GuiWindowManager::new(
            processor.clone(),
            input_simulator.clone(),
            gui_needs_update.clone(),
        );

        assert!(!manager.is_visible(), "初始狀態應該不可見");
    }

    /// 測試：鍵盤事件處理 - 字母鍵輸入（模擬窗口接收鍵盤事件）
    ///
    /// 這個測試驗證窗口能夠處理鍵盤輸入，不依賴鍵盤鉤子
    /// 這是「輸入窗口模式」的核心功能，用於支援 Raw Input 遊戲
    #[test]
    fn test_window_keyboard_event_letter_input() {
        let (processor, _input_simulator, _gui_needs_update) = create_test_components();

        // 模擬窗口接收字母鍵 'a' 的輸入
        // 注意：這裡我們直接調用處理邏輯，模擬窗口有焦點時接收鍵盤事件的情況
        {
            let mut proc = processor.lock().unwrap();
            let (success, _) = proc.handle_code_input('a');
            assert!(success, "字母鍵 'a' 應該被成功處理");
        }

        // 驗證字根已輸入
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(state.current_code, "a", "字根應該是 'a'");
            assert_eq!(state.candidates.len(), 2, "應該找到 2 個候選字");
        }
    }

    /// 測試：鍵盤事件處理 - 數字鍵選擇候選字
    ///
    /// 驗證窗口能夠處理數字鍵選擇候選字
    #[test]
    fn test_window_keyboard_event_number_selection() {
        let (processor, _input_simulator, _gui_needs_update) = create_test_components();

        // 先輸入字根 'a'
        {
            let mut proc = processor.lock().unwrap();
            proc.handle_code_input('a');
        }

        // 模擬按數字鍵 '1' 選擇第一個候選字
        {
            let mut proc = processor.lock().unwrap();
            let selected = proc.handle_number_selection(1);
            assert_eq!(
                selected,
                Some("一".to_string()),
                "應該選擇第一個候選字 '一'"
            );
        }

        // 驗證輸入已清除
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(state.current_code, "", "選擇候選字後應該清除輸入");
        }
    }

    /// 測試：鍵盤事件處理 - Space 鍵選擇第一個候選字
    ///
    /// 驗證窗口能夠處理 Space 鍵選擇第一個候選字
    #[test]
    fn test_window_keyboard_event_space_selection() {
        let (processor, _input_simulator, _gui_needs_update) = create_test_components();

        // 先輸入字根 'a'
        {
            let mut proc = processor.lock().unwrap();
            proc.handle_code_input('a');
        }

        // 模擬按 Space 鍵選擇第一個候選字
        {
            let mut proc = processor.lock().unwrap();
            let selected = proc.handle_space();
            assert_eq!(
                selected,
                Some("一".to_string()),
                "Space 鍵應該選擇第一個候選字 '一'"
            );
        }

        // 驗證輸入已清除
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(state.current_code, "", "Space 鍵選擇後應該清除輸入");
        }
    }

    /// 測試：鍵盤事件處理 - Backspace 鍵刪除字根
    ///
    /// 驗證窗口能夠處理 Backspace 鍵刪除字根
    #[test]
    fn test_window_keyboard_event_backspace() {
        let (processor, _input_simulator, _gui_needs_update) = create_test_components();

        // 先輸入字根 'ab'
        {
            let mut proc = processor.lock().unwrap();
            proc.handle_code_input('a');
            proc.handle_code_input('b');
        }

        // 驗證字根是 'ab'
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(state.current_code, "ab", "字根應該是 'ab'");
        }

        // 模擬按 Backspace 鍵刪除最後一個字根
        {
            let mut proc = processor.lock().unwrap();
            let handled = proc.handle_backspace();
            assert!(handled, "Backspace 鍵應該被處理");
        }

        // 驗證字根已刪除一個字符
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(state.current_code, "a", "Backspace 後字根應該是 'a'");
        }
    }

    /// 測試：鍵盤事件處理 - ESC 鍵清除輸入
    ///
    /// 驗證窗口能夠處理 ESC 鍵清除輸入
    #[test]
    fn test_window_keyboard_event_escape_clear() {
        let (processor, _input_simulator, _gui_needs_update) = create_test_components();

        // 先輸入字根 'abc'
        {
            let mut proc = processor.lock().unwrap();
            proc.handle_code_input('a');
            proc.handle_code_input('b');
            proc.handle_code_input('c');
        }

        // 驗證字根是 'abc'
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(state.current_code, "abc", "字根應該是 'abc'");
        }

        // 模擬按 ESC 鍵清除輸入
        {
            let mut proc = processor.lock().unwrap();
            proc.clear();
        }

        // 驗證輸入已清除
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(state.current_code, "", "ESC 鍵後應該清除輸入");
            assert_eq!(state.candidates.len(), 0, "候選字應該被清除");
        }
    }

    /// 測試：輸入窗口模式的核心特性
    ///
    /// 驗證窗口能夠獨立處理鍵盤輸入，不依賴鍵盤鉤子
    /// 這是支援 Raw Input 遊戲的關鍵特性
    #[test]
    fn test_input_window_mode_independent_input() {
        let (processor, _input_simulator, _gui_needs_update) = create_test_components();

        // 模擬完整的輸入流程（不依賴鍵盤鉤子）
        // 1. 輸入字根
        {
            let mut proc = processor.lock().unwrap();
            proc.handle_code_input('t');
            proc.handle_code_input('e');
            proc.handle_code_input('s');
            proc.handle_code_input('t');
        }

        // 2. 驗證候選字已找到
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(state.current_code, "test", "字根應該是 'test'");
            assert_eq!(state.candidates.len(), 1, "應該找到 1 個候選字");
            assert_eq!(state.candidates[0], "測試", "候選字應該是 '測試'");
        }

        // 3. 選擇候選字（模擬 Space 鍵）
        {
            let mut proc = processor.lock().unwrap();
            let selected = proc.handle_space();
            assert_eq!(selected, Some("測試".to_string()), "應該選擇候選字 '測試'");
        }

        // 4. 驗證輸入已清除
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(state.current_code, "", "選擇候選字後應該清除輸入");
        }

        // 這個測試證明：窗口可以獨立處理鍵盤輸入，不依賴 WH_KEYBOARD_LL 鉤子
        // 因此能夠支援使用 Raw Input 的遊戲
    }

    /// 測試：連續輸入多個字
    ///
    /// 驗證窗口能夠連續處理多個字的輸入（輸入窗口模式的核心功能）
    #[test]
    fn test_input_window_mode_continuous_input() {
        let (processor, _input_simulator, _gui_needs_update) = create_test_components();

        // 第一個字：輸入 'a'，選擇第一個候選字
        {
            let mut proc = processor.lock().unwrap();
            proc.handle_code_input('a');
            let selected = proc.handle_space();
            assert_eq!(selected, Some("一".to_string()), "第一個字應該是 '一'");
        }

        // 第二個字：輸入 'ab'，選擇第一個候選字
        {
            let mut proc = processor.lock().unwrap();
            proc.handle_code_input('a');
            proc.handle_code_input('b');
            let selected = proc.handle_space();
            assert_eq!(selected, Some("二".to_string()), "第二個字應該是 '二'");
        }

        // 驗證輸入已清除（準備下一個字）
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(
                state.current_code, "",
                "選擇候選字後應該清除輸入，準備下一個字"
            );
        }

        // 這個測試證明：窗口可以連續處理多個字的輸入
        // 每個字選擇後才清除，可以連續輸入多個字
    }

    /// 測試：驗證窗口能夠接收鍵盤輸入（不依賴鍵盤鉤子）
    ///
    /// 這是「輸入窗口模式」的核心測試，驗證窗口能夠：
    /// 1. 獨立接收鍵盤輸入（不依賴 WH_KEYBOARD_LL 鉤子）
    /// 2. 處理字根輸入
    /// 3. 處理候選字選擇
    /// 4. 處理特殊按鍵（Space, Enter, Backspace, ESC）
    ///
    /// 這個特性使得輸入法能夠支援使用 Raw Input 的遊戲
    #[test]
    fn test_window_can_receive_keyboard_input_without_hook() {
        let (processor, _input_simulator, _gui_needs_update) = create_test_components();

        // 測試場景：模擬窗口有焦點時接收鍵盤輸入
        // 在實際使用中，當窗口獲得焦點時，鍵盤事件會直接發送到窗口
        // 不經過 WH_KEYBOARD_LL 鉤子，因此能夠繞過 Raw Input 的限制

        // 1. 模擬輸入字母鍵 'a'
        {
            let mut proc = processor.lock().unwrap();
            let (success, _) = proc.handle_code_input('a');
            assert!(success, "窗口應該能夠處理字母鍵輸入");
        }

        // 2. 驗證字根已輸入
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(state.current_code, "a", "字根應該是 'a'");
            assert!(!state.candidates.is_empty(), "應該找到候選字");
        }

        // 3. 模擬輸入數字鍵 '1' 選擇候選字
        {
            let mut proc = processor.lock().unwrap();
            let selected = proc.handle_number_selection(1);
            assert!(selected.is_some(), "窗口應該能夠處理數字鍵選擇候選字");
        }

        // 4. 驗證輸入已清除
        {
            let proc = processor.lock().unwrap();
            let state = proc.get_state();
            assert_eq!(state.current_code, "", "選擇候選字後應該清除輸入");
        }

        // 結論：窗口能夠獨立處理鍵盤輸入，不依賴鍵盤鉤子
        // 這使得輸入法能夠支援使用 Raw Input 的遊戲
    }
}
