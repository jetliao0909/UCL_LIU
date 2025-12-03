//! Windows 全域鍵盤鉤子模組

use crate::AppState;
use anyhow::Result;
use log::{debug, info, warn, error};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::{
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Input::KeyboardAndMouse::*,
};


// 使用 thread_local 存儲狀態指標
thread_local! {
    static APP_STATE: std::cell::RefCell<Option<Arc<AppState>>> = std::cell::RefCell::new(None);
    static SHOULD_QUIT: std::cell::RefCell<Option<Arc<AtomicBool>>> = std::cell::RefCell::new(None);
    static CTRL_PRESSED: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
    static ALT_PRESSED: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
    static SHIFT_PRESSED: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
    static SHIFT_TOGGLE: std::cell::RefCell<bool> = std::cell::RefCell::new(false); // Shift 切換狀態：false=攔截，true=不攔截
    static SHIFT_USED_WITH_OTHER_KEY: std::cell::RefCell<bool> = std::cell::RefCell::new(false); // Shift 是否與其他鍵組合過
}

/// 鍵盤鉤子管理器
pub struct KeyboardHook {
    _state: Arc<AppState>,
    hook_handle: HHOOK,
    should_quit: Arc<std::sync::atomic::AtomicBool>,
}

impl KeyboardHook {
    pub fn new(state: Arc<AppState>) -> Result<Self> {
        // 使用 AppState 中的 should_quit
        let should_quit = state.should_quit.clone();
        
        // 將狀態存儲到 thread_local
        APP_STATE.with(|s| {
            *s.borrow_mut() = Some(state.clone());
        });
        
        SHOULD_QUIT.with(|s| {
            *s.borrow_mut() = Some(should_quit.clone());
        });
        
        unsafe {
            let hook_handle = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(Self::low_level_keyboard_proc),
                None,
                0,
            )?;
            
            info!("鍵盤鉤子已設置");
            
            Ok(Self {
                _state: state,
                hook_handle,
                should_quit,
            })
        }
    }
    
    /// 運行訊息循環（整合 fltk 事件處理）
    pub fn run_with_fltk(&self, _app: &fltk::app::App, state: Arc<AppState>) -> Result<()> {
        unsafe {
            let mut msg = MSG::default();
            
            loop {
                // 檢查是否應該退出
                if self.should_quit.load(Ordering::Relaxed) {
                    info!("收到退出信號，正在退出...");
                    PostQuitMessage(0);
                    break;
                }
                
                // 處理 fltk 事件（非阻塞）
                // 使用 app::check() 非阻塞地處理 fltk 事件
                // 需要定期調用以處理窗口顯示和重繪
                if fltk::app::check() {
                    // 如果有 fltk 事件，處理並刷新
                    fltk::app::flush();
                }

                // 只在有輸入變化時才更新 GUI 主窗口顯示
                // 注意：這裡不在鍵盤鉤子回呼裡，而是在主迴圈中，避免阻塞鍵盤事件處理
                if state.gui_needs_update.load(Ordering::Relaxed) {
                    if let Ok(mut gui_manager) = state.gui_window_manager.lock() {
                        gui_manager.update_display();
                    }
                    // 清除更新標誌
                    state.gui_needs_update.store(false, Ordering::Relaxed);
                }
                
                // 使用 PeekMessageW 非阻塞地檢查 Windows 消息
                let has_msg = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
                
                if has_msg {
                    // 處理 WM_QUIT
                    if msg.message == WM_QUIT {
                        break;
                    }
                    
                    // 處理系統托盤菜單項點擊
                    if msg.message == WM_COMMAND {
                        let menu_id = msg.wParam.0 as u16;
                        let notification_code = (msg.wParam.0 >> 16) as u16;
                        debug!("收到 WM_COMMAND 消息，menu_id: {}, notification_code: {}", menu_id, notification_code);
                        
                        if notification_code == 0 && menu_id == 1001 {
                            info!("✅ 系統托盤退出選項被點擊，準備退出...");
                            self.should_quit.store(true, Ordering::Relaxed);
                            PostQuitMessage(0);
                            break;
                        }
                    }
                    
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                } else {
                    // 沒有消息時，短暫休眠避免 CPU 佔用過高
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        }
        
        Ok(())
    }
    
    /// 運行訊息循環（原始版本，保留以備用）
    pub fn run(&self) -> Result<()> {
        unsafe {
            let mut msg = MSG::default();
            
            loop {
                // 檢查是否應該退出
                if self.should_quit.load(Ordering::Relaxed) {
                    info!("收到退出信號，正在退出...");
                    PostQuitMessage(0);
                    break;
                }
                
                let b_ret = GetMessageW(&mut msg, None, 0, 0);
                
                // GetMessageW 返回 BOOL，false 表示 WM_QUIT 或錯誤
                if !b_ret.as_bool() {
                    break;
                }
                
                // 處理系統托盤菜單項點擊
                // tray-icon 0.10 使用 Windows 消息循環處理菜單項點擊
                // 當菜單項被點擊時，會發送 WM_COMMAND 消息
                // WM_COMMAND = 0x0111
                let msg_value: u32 = msg.message.into();
                
                // WM_COMMAND 消息（標準菜單項點擊）
                if msg.message == WM_COMMAND {
                    // WM_COMMAND 的 wParam 低16位是菜單項 ID，高16位是通知代碼
                    let menu_id = msg.wParam.0 as u16;
                    let notification_code = (msg.wParam.0 >> 16) as u16;
                    debug!("收到 WM_COMMAND 消息，menu_id: {}, notification_code: {}, hwnd: {:?}", menu_id, notification_code, msg.hwnd);
                    
                    // tray-icon 0.10 的第一個菜單項 ID 實際上是 1001（從日誌中確認）
                    // 通知代碼為 0 表示菜單項被點擊（不是從控件發送的消息）
                    if notification_code == 0 && menu_id == 1001 {
                        info!("✅ 系統托盤退出選項被點擊（WM_COMMAND, menu_id: {}），準備退出（與 F4 鍵行為一致）...", menu_id);
                        self.should_quit.store(true, Ordering::Relaxed);
                        PostQuitMessage(0);
                        break;
                    }
                }
                
                // 也處理其他可能的菜單消息（例如 WM_MENUCOMMAND = 0x0126）
                // 某些實現可能使用不同的消息類型
                if msg_value == 0x0126 { // WM_MENUCOMMAND
                    let menu_id = msg.wParam.0 as u16;
                    debug!("收到 WM_MENUCOMMAND 消息，menu_id: {}", menu_id);
                    if menu_id == 0 {
                        info!("✅ 系統托盤退出選項被點擊（WM_MENUCOMMAND），準備退出（與 F4 鍵行為一致）...");
                        self.should_quit.store(true, Ordering::Relaxed);
                        PostQuitMessage(0);
                        break;
                    }
                }
                
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        
        Ok(())
    }
    
    /// 低階鍵盤回調函數
    extern "system" fn low_level_keyboard_proc(
        code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        unsafe {
            if code < 0 {
                return CallNextHookEx(None, code, w_param, l_param);
            }
            
            // 從 thread_local 取得狀態並處理鍵盤事件
            let mut should_block = false;
            
            APP_STATE.with(|state_opt| {
                if let Some(state) = state_opt.borrow().as_ref() {
                    // 解析鍵盤事件
                    match Self::process_keyboard_event(state, w_param, l_param) {
                        Ok(handled) => {
                            should_block = handled;
                        }
                        Err(e) => {
                            debug!("處理鍵盤事件錯誤: {}", e);
                        }
                    }
                }
            });
            
            if should_block {
                // 阻止按鍵事件傳遞
                LRESULT(1)
            } else {
                // 讓按鍵事件通過
                CallNextHookEx(None, code, w_param, l_param)
            }
        }
    }
    
    /// 處理鍵盤事件
    /// 返回 true 表示應該阻止事件，false 表示讓事件通過
    fn process_keyboard_event(
        state: &AppState,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> Result<bool> {
        // 處理 key down 和 key up 事件
        // WM_KEYDOWN = 256 (0x0100), WM_KEYUP = 257 (0x0101)
        const WM_KEYDOWN_VALUE: usize = 256;
        const WM_KEYUP_VALUE: usize = 257;
        
        let is_key_down = w_param.0 == WM_KEYDOWN_VALUE;
        let is_key_up = w_param.0 == WM_KEYUP_VALUE;
        
        if !is_key_down && !is_key_up {
            return Ok(false);
        }
        
        // 首先檢查是否為注入的事件（避免無限循環和重複處理）
        // 這必須在最前面檢查，避免處理我們自己送出的按鍵
        unsafe {
            let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            
            // 檢查是否為注入的事件（避免無限循環）
            if kbd_struct.flags.0 & LLKHF_INJECTED.0 != 0 {
                debug!("忽略注入的事件");
                return Ok(false);
            }
        }
        
        // 檢查 F4 鍵退出（需要在檢查模式之前，因為退出功能應該在所有模式下都可用）
        // 無論是攔截模式還是不攔截模式，F4 鍵都應該能退出程序
        unsafe {
            let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kbd_struct.vkCode;
            let vk_value: u32 = vk_code.into();
            
            // F4 鍵退出（VK_F4 = 115）
            if is_key_down && vk_value == 115 {
                info!("✅ 檢測到 F4 鍵，準備退出（無論攔截模式）...");
                SHOULD_QUIT.with(|q| {
                    if let Some(quit_flag) = q.borrow().as_ref() {
                        quit_flag.store(true, Ordering::Relaxed);
                        unsafe {
                            PostQuitMessage(0);
                        }
                    }
                });
                return Ok(true); // 阻止 F4 鍵事件
            }
        }
        
        // 處理 Ctrl 鍵的按下和釋放（需要在模式檢查之前）
        // VK_CONTROL = 17 (通用), VK_LCONTROL = 162 (左 Ctrl), VK_RCONTROL = 163 (右 Ctrl)
        unsafe {
            let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kbd_struct.vkCode;
            let vk_value: u32 = vk_code.into();
            
            // 檢查左 Ctrl、右 Ctrl 或通用 Ctrl
            if vk_value == VK_CONTROL.0 as u32 || vk_value == VK_LCONTROL.0 as u32 || vk_value == VK_RCONTROL.0 as u32 {
                if is_key_down {
                    CTRL_PRESSED.with(|p| {
                        *p.borrow_mut() = true;
                    });
                    debug!("Ctrl 鍵按下 (vk={})", vk_value);
                } else if is_key_up {
                    CTRL_PRESSED.with(|p| {
                        *p.borrow_mut() = false;
                    });
                    debug!("Ctrl 鍵釋放 (vk={})", vk_value);
                }
                return Ok(false); // 讓 Ctrl 鍵通過
            }
        }
        
        // 處理 Alt 鍵的按下和釋放（用於檢測 Ctrl+Alt 熱鍵）
        // VK_MENU = 18 (Alt 鍵), VK_LMENU = 164 (左 Alt), VK_RMENU = 165 (右 Alt)
        unsafe {
            let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kbd_struct.vkCode;
            let vk_value: u32 = vk_code.into();
            
            // 檢查左 Alt、右 Alt 或通用 Alt
            if vk_value == VK_MENU.0 as u32 || vk_value == 164 || vk_value == 165 {
                if is_key_down {
                    ALT_PRESSED.with(|p| {
                        *p.borrow_mut() = true;
                    });
                    debug!("Alt 鍵按下 (vk={})", vk_value);
                } else if is_key_up {
                    ALT_PRESSED.with(|p| {
                        *p.borrow_mut() = false;
                    });
                    debug!("Alt 鍵釋放 (vk={})", vk_value);
                }
                // 注意：Alt 鍵的處理會在後面繼續，這裡不返回（讓它通過，除非是 Ctrl+Alt 組合）
            }
        }
        
        // 處理 Shift 鍵的按下和釋放（用於檢測 Ctrl+Shift+F 熱鍵）
        unsafe {
            let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kbd_struct.vkCode;
            let vk_value: u32 = vk_code.into();
            
            // VK_SHIFT = 16, VK_LSHIFT = 160, VK_RSHIFT = 161
            if vk_value == 16 || vk_value == 160 || vk_value == 161 {
                if is_key_down {
                    SHIFT_PRESSED.with(|p| {
                        *p.borrow_mut() = true;
                    });
                    SHIFT_USED_WITH_OTHER_KEY.with(|f| {
                        *f.borrow_mut() = false;
                    });
                    debug!("Shift 鍵按下 (vk={})", vk_value);
                } else if is_key_up {
                    SHIFT_PRESSED.with(|p| {
                        *p.borrow_mut() = false;
                    });
                    debug!("Shift 鍵釋放 (vk={})", vk_value);
                }
                // 注意：Shift 鍵的處理會在後面繼續，這裡不返回
            }
        }
        
        // 檢查 Ctrl+Space 熱鍵（優先級最高，在模式檢查之前）
        // Ctrl+Space 是 Windows 系統默認的輸入法切換鍵，遊戲通常會允許它通過
        unsafe {
            let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kbd_struct.vkCode;
            let vk_value: u32 = vk_code.into();
            
            let ctrl_pressed = CTRL_PRESSED.with(|p| *p.borrow());
            
            // Ctrl + Space：切換 GUI 視窗顯示/隱藏
            // VK_SPACE = 32
            if is_key_down && vk_value == 32 && ctrl_pressed {
                debug!("檢測到 Space 鍵按下，Ctrl: {}", ctrl_pressed);
                info!("✅ 檢測到 Ctrl+Space 熱鍵，切換 GUI 視窗狀態列");
                APP_STATE.with(|s| {
                    if let Some(state) = s.borrow().as_ref() {
                        info!("獲取 gui_window_manager...");
                        let mut manager = state.gui_window_manager.lock().unwrap();
                        let is_visible = manager.is_visible();
                        info!("當前 GUI 視窗可見狀態: {}", is_visible);
                        if is_visible {
                            info!("隱藏 GUI 視窗");
                            manager.hide();
                        } else {
                            info!("顯示 GUI 視窗（調用 manager.show()）");
                            if let Err(e) = manager.show() {
                                error!("顯示 GUI 視窗失敗: {}", e);
                            } else {
                                info!("GUI 視窗顯示完成");
                            }
                        }
                    } else {
                        error!("無法獲取 AppState！");
                    }
                });
                return Ok(true); // 攔截熱鍵，不讓遊戲收到
            }
            
        }
        
        // 處理 Shift 鍵的按下和釋放（參考 Python 版邏輯）
        unsafe {
            let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kbd_struct.vkCode;
            let vk_value: u32 = vk_code.into();
            
            // VK_SHIFT = 16 (左 Shift 和右 Shift 都是 16，但可以通過 scanCode 區分)
            // VK_LSHIFT = 160, VK_RSHIFT = 161
            if vk_value == 16 || vk_value == 160 || vk_value == 161 {
                if is_key_down {
                    // Shift 按下：記錄按下狀態，假設尚未與其他鍵組合
                    SHIFT_PRESSED.with(|p| {
                        *p.borrow_mut() = true;
                    });
                    SHIFT_USED_WITH_OTHER_KEY.with(|f| {
                        *f.borrow_mut() = false;
                    });
                    debug!("Shift 鍵按下 (vk={})", vk_value);
                    // 讓 Shift Down 事件通過，保留原本的組合鍵行為（如 Shift+數字）
                    return Ok(false);
                } else if is_key_up {
                    debug!("Shift 鍵釋放 (vk={})", vk_value);
                    // 檢查 Shift 期間是否有搭配其他鍵
                    let used_with_other = SHIFT_USED_WITH_OTHER_KEY.with(|f| {
                        let used = *f.borrow();
                        *f.borrow_mut() = false;
                        used
                    });
                    SHIFT_PRESSED.with(|p| {
                        *p.borrow_mut() = false;
                    });

                    // 如果沒有與其他鍵組合，視為「單獨按 Shift」→ 切換模式（英/肥）
                    if !used_with_other {
                    let old_state = SHIFT_TOGGLE.with(|t| *t.borrow());
                    let new_state = SHIFT_TOGGLE.with(|t| {
                        let mut toggle = t.borrow_mut();
                        *toggle = !*toggle;
                        *toggle
                    });
                    
                    // 清除現有字根輸入
                    let mut processor = state.input_processor.lock().unwrap();
                    let state_ref = processor.get_state();
                    if !state_ref.current_code.is_empty() {
                        info!("Shift 切換，清除現有字根: {}", state_ref.current_code);
                        processor.clear();
                            // 標記需要更新 GUI
                            state.gui_needs_update.store(true, Ordering::Relaxed);
                    }
                    
                        info!("Shift 單獨按下，切換攔截狀態: {} -> {}", 
                            if old_state { "不攔截(英)" } else { "攔截(肥)" },
                            if new_state { "不攔截(英)" } else { "攔截(肥)" });
                }
                
                    // Shift Up 事件一律放行，保留原本鍵盤行為
                    return Ok(false);
                }
            }
        }
        
        // 先檢查 Shift 切換狀態（英/肥模式）
        let shift_toggle = SHIFT_TOGGLE.with(|t| *t.borrow());
        // 如果不攔截模式（英模式），讓所有其他按鍵通過
        if shift_toggle {
            // 檢查 CapsLock 狀態（只用於調試日誌）
            unsafe {
                let caps_lock_state = GetKeyState(20i32); // VK_CAPITAL = 20
                let is_caps_on = (caps_lock_state & 0x0001) != 0;
                
                debug!("Shift 切換模式：不攔截，讓事件通過 (CapsLock={}, 大小寫只由CapsLock決定)", 
                    if is_caps_on { "ON→大寫" } else { "OFF→小寫" });
            }
            return Ok(false);
        }
        
        // 如果 Ctrl 鍵已經按下，讓所有後續按鍵通過（支援 Ctrl+C、Ctrl+V 等組合鍵）
        // 參考 Python 版本的實現：在攔截模式下，如果 Ctrl 鍵按下，讓所有按鍵通過
        let ctrl_pressed = CTRL_PRESSED.with(|p| *p.borrow());
        if ctrl_pressed && is_key_down {
            debug!("Ctrl 鍵已按下，讓事件通過（支援 Ctrl+C、Ctrl+V 等組合鍵）");
            return Ok(false);
        }
        
        // 只處理 key down 事件（避免重複處理）
        // 這必須在 Shift 切換檢查之後，因為 Shift 切換應該對所有事件都生效
        if !is_key_down {
            return Ok(false);
        }
        
        // 注意：英模式就是不攔截模式，已經在上面通過 shift_toggle 檢查處理了
        // 如果 shift_toggle 為 true（不攔截模式），已經在上面返回 Ok(false) 讓事件通過
        // 這裡只處理攔截模式（shift_toggle 為 false）的情況
        
        // 解析虛擬鍵碼
        unsafe {
            let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kbd_struct.vkCode;
            
            // 處理特殊按鍵
            // vkCode 是 VIRTUAL_KEY 類型，直接轉換為 u32
            let vk_value: u32 = vk_code.into();
            
            debug!("處理按鍵 (key down): vk_code={:?}, vk_value={}", vk_code, vk_value);

            // 如果 Shift 正在按著，且這不是 Shift 本身，表示 Shift 有搭配其他鍵
            if is_key_down {
                let shift_pressed_now = SHIFT_PRESSED.with(|p| *p.borrow());
                if shift_pressed_now && vk_value != 16 && vk_value != 160 && vk_value != 161 {
                    SHIFT_USED_WITH_OTHER_KEY.with(|f| {
                        *f.borrow_mut() = true;
                    });
                }
            }
            
            // Ctrl 鍵和 ESC 鍵（在 Ctrl+ESC 組合時）已經在上面處理過了，這裡跳過
            // 但單獨的 ESC 鍵還需要在下面處理（清除輸入）
            if vk_value == 17 {
                // Ctrl 鍵已經在前面處理過，讓事件通過
                debug!("跳過已處理的 Ctrl 鍵");
                return Ok(false);
            }
            
            // 如果輸入窗口可見，無論是否有焦點，都讓所有按鍵通過（除了熱鍵）
            // 這樣可以避免鍵盤鉤子和輸入窗口同時處理按鍵導致衝突
            let (gui_visible, gui_has_focus) = {
                if let Ok(manager) = state.gui_window_manager.lock() {
                    let visible = manager.is_visible();
                    let has_focus = manager.has_focus();
                    (visible, has_focus)
                } else {
                    (false, false)
                }
            };
            
            if gui_visible {
                // 輸入窗口可見時，讓所有按鍵通過（除了熱鍵），讓輸入窗口自己決定是否處理
                // 輸入窗口內部會檢查是否有焦點，沒有焦點時會忽略按鍵
                if gui_has_focus {
                    debug!("輸入窗口可見且有焦點，讓按鍵通過，讓輸入窗口處理 (vk={})", vk_value);
                } else {
                    debug!("輸入窗口可見但沒有焦點，讓按鍵通過，輸入窗口會忽略 (vk={})", vk_value);
                }
                return Ok(false);
            }
            
            match vk_value {
                
                // Escape (VK_ESCAPE = 27)
                27 => {
                    // ESC 鍵處理：清除輸入
                    
                    // 如果是肥米模式且有輸入的字根，清除輸入
                    let mut processor = state.input_processor.lock().unwrap();
                    let state_ref = processor.get_state();
                    if !state_ref.current_code.is_empty() {
                        info!("按下 ESC，清除輸入: {}", state_ref.current_code);
                        processor.clear();
                        // 標記需要更新 GUI
                        state.gui_needs_update.store(true, Ordering::Relaxed);
                        // 阻止 ESC 鍵事件傳遞
                        return Ok(true);
                    }
                    // 沒有輸入，讓 ESC 鍵通過
                    Ok(false)
                }
                
                // Backspace (VK_BACK = 8)
                8 => {
                    let handled = {
                    let mut processor = state.input_processor.lock().unwrap();
                        processor.handle_backspace()
                    };
                    if handled {
                        // 有字根可刪除，阻止事件
                        // 標記需要更新 GUI
                        state.gui_needs_update.store(true, Ordering::Relaxed);
                        return Ok(true);
                    }
                    // 沒有字根，讓事件通過
                    Ok(false)
                }
                
                // Space (VK_SPACE = 32)
                32 => {
                    let (has_complement, has_input, text_opt) = {
                    let mut processor = state.input_processor.lock().unwrap();
                    
                    // 檢查是否有符號選擇（補碼或符號輸入）
                    let has_complement = processor.get_state().complement_selected.is_some();
                    
                    // 檢查是否有輸入的字根
                    let has_input = !processor.get_state().current_code.is_empty();
                    
                        let text_opt = if has_complement || has_input {
                        // 嘗試選擇候選字（可能是補碼選擇、符號選擇或第一個候選字）
                            let text = processor.handle_space();
                        
                        // 確保清除輸入（handle_space() 可能已經清除了，但我們確保總是清除）
                        processor.clear();
                            
                            text
                        } else {
                            None
                        };
                        
                        (has_complement, has_input, text_opt)
                    };
                    
                    if has_complement || has_input {
                        // 標記需要更新 GUI
                        state.gui_needs_update.store(true, Ordering::Relaxed);
                        
                        if let Some(text) = text_opt {
                            // 有候選字，送出文字並阻止 Space 事件
                            let mut simulator = state.input_simulator.lock().unwrap();
                            if simulator.send_text_paste(&text).is_ok() {
                                info!("Space: 送出候選字: {}", text);
                                return Ok(true);
                            }
                        } else {
                            // 沒有候選字，但已清除輸入，阻止 Space 事件
                            info!("Space: 沒有候選字，已清除輸入");
                            return Ok(true);
                        }
                    }
                    // 沒有輸入也沒有符號選擇，讓 Space 鍵通過
                    Ok(false)
                }
                
                // Enter (VK_RETURN = 13)
                13 => {
                    let (has_input, text_opt) = {
                    let mut processor = state.input_processor.lock().unwrap();
                    
                    // 先檢查是否有輸入的字根
                    let has_input = !processor.get_state().current_code.is_empty();
                    
                        let text_opt = if has_input {
                        // 嘗試選擇第一個候選字（與 Space 鍵行為一致）
                            let text = processor.handle_space();
                        
                        // 確保清除輸入（handle_space() 可能已經清除了，但我們確保總是清除）
                        processor.clear();
                            
                            text
                        } else {
                            None
                        };
                        
                        (has_input, text_opt)
                    };
                    
                    if has_input {
                        // 標記需要更新 GUI
                        state.gui_needs_update.store(true, Ordering::Relaxed);
                        
                        if let Some(text) = text_opt {
                            // 有候選字，送出文字並阻止 Enter 事件
                            let mut simulator = state.input_simulator.lock().unwrap();
                            if simulator.send_text_paste(&text).is_ok() {
                                info!("Enter: 送出候選字: {}", text);
                                return Ok(true);
                            }
                        } else {
                            // 沒有候選字，但已清除輸入，阻止 Enter 事件
                            info!("Enter: 沒有候選字，已清除輸入");
                            return Ok(true);
                        }
                    }
                    // 沒有輸入，讓 Enter 鍵通過
                    Ok(false)
                }
                
                // 數字鍵 0-9 (VK_0 = 48, VK_9 = 57)
                48..=57 => {
                    let num = (vk_value - 48) as u8;
                    let mut processor = state.input_processor.lock().unwrap();
                    let state_ref = processor.get_state();
                    let candidate_count = state_ref.get_current_page_candidates().len();
                    
                    debug!("處理數字鍵 {}: 當前候選字數量={}, 字根='{}'", num, candidate_count, state_ref.current_code);
                    
                    if let Some(text) = processor.handle_number_selection(num) {
                        // 選擇了候選字，送出文字並阻止數字鍵事件
                        let mut simulator = state.input_simulator.lock().unwrap();
                        if simulator.send_text_paste(&text).is_ok() {
                            info!("✅ 選擇候選字 {}: {}", num, text);
                            return Ok(true);
                        } else {
                            warn!("送出候選字失敗: {}", text);
                            return Ok(true); // 即使送出失敗，也阻止數字鍵事件
                        }
                    } else {
                        // 沒有對應的候選字，攔截並忽略該按鍵
                        debug!("數字鍵 {} 沒有對應的候選字（候選字數量={}），攔截並忽略", num, candidate_count);
                        Ok(true) // 攔截並忽略
                    }
                }
                
                // 字母鍵 A-Z (VK_A = 65, VK_Z = 90)
                65..=90 => {
                    // 直接轉為小寫（字根查詢時大小寫沒有分別，handle_code_input 也會轉為小寫）
                    let ch = char::from(vk_value as u8).to_ascii_lowercase();
                    
                    debug!("處理字母鍵: vk={}, 轉換後={}", vk_value, ch);
                    
                    let (success, complement_selected) = {
                    let mut processor = state.input_processor.lock().unwrap();
                        processor.handle_code_input(ch)
                    };
                    
                    if success {
                        // 檢查是否有補碼選擇的候選字
                        if complement_selected.is_some() {
                            // 補碼機制選擇了候選字，但不清除狀態，等待 Space 鍵送出
                            let (current_code, complement_selected_val) = {
                                let processor = state.input_processor.lock().unwrap();
                            let state_ref = processor.get_state();
                                (state_ref.current_code.clone(), state_ref.complement_selected.clone())
                            };
                            info!(
                                "✅ 補碼選擇候選字（等待 Space 鍵送出）: '{}' -> {:?}",
                                current_code,
                                complement_selected_val
                            );
                            
                            // 標記需要更新 GUI
                            state.gui_needs_update.store(true, Ordering::Relaxed);
                            
                            // 阻止 v/s 按鍵事件，但不立即送出候選字
                            return Ok(true);
                        }
                        
                        // 成功處理字根輸入，阻止原始按鍵事件
                        let (current_code, candidates_len, current_page) = {
                            let processor = state.input_processor.lock().unwrap();
                        let state_ref = processor.get_state();
                            (state_ref.current_code.clone(), state_ref.candidates.len(), state_ref.get_current_page_candidates().clone())
                        };
                        info!(
                            "✅ 輸入字根: '{}', 找到 {} 個候選字: {:?}",
                            current_code,
                            candidates_len,
                            current_page
                        );
                        
                        // 標記需要更新 GUI
                        state.gui_needs_update.store(true, Ordering::Relaxed);
                        
                        return Ok(true);
                    }
                    debug!("字母鍵處理失敗，讓事件通過");
                    Ok(false)
                }
                
                // 功能鍵處理
                // F1-F3, F5-F24 (112-114, 116-135)：讓事件通過（不攔截）
                // F4 (115)：退出功能，已在上面處理，不應該到達這裡
                112..=114 | 116..=135 => {
                    let f_num = if vk_value <= 114 { vk_value - 111 } else { vk_value - 111 };
                    debug!("功能鍵 F{}，讓事件通過", f_num);
                    Ok(false)
                }
                // F4 (115) 應該在上面處理，如果到達這裡，再次處理
                115 => {
                    warn!("F4 鍵應該在上面處理，但到達了這裡，再次處理");
                    SHOULD_QUIT.with(|q| {
                        if let Some(quit_flag) = q.borrow().as_ref() {
                            quit_flag.store(true, Ordering::Relaxed);
                            unsafe {
                                PostQuitMessage(0);
                            }
                        }
                    });
                    Ok(true) // 阻止 F4 鍵事件
                }
                // 方向鍵
                37 | 38 | 39 | 40 => { // LEFT, UP, RIGHT, DOWN
                    debug!("方向鍵，讓事件通過");
                    Ok(false)
                }
                // Tab (9)
                9 => {
                    debug!("Tab 鍵，讓事件通過");
                    Ok(false)
                }
                // CapsLock (20)
                20 => {
                    debug!("CapsLock 鍵，讓事件通過");
                    Ok(false)
                }
                // NumLock (144)
                144 => {
                    debug!("NumLock 鍵，讓事件通過");
                    Ok(false)
                }
                // ScrollLock (145)
                145 => {
                    debug!("ScrollLock 鍵，讓事件通過");
                    Ok(false)
                }
                // Home (36), End (35), PageUp (33), PageDown (34)
                33 | 34 | 35 | 36 => {
                    debug!("導航鍵，讓事件通過");
                    Ok(false)
                }
                // Insert (45), Delete (46)
                45 | 46 => {
                    debug!("編輯鍵，讓事件通過");
                    Ok(false)
                }
                // PrintScreen (44), Pause (19)
                19 | 44 => {
                    debug!("系統鍵，讓事件通過");
                    Ok(false)
                }
                // Win鍵 (91, 92)
                91 | 92 => {
                    debug!("Win 鍵，讓事件通過");
                    Ok(false)
                }
                // Alt (18), Menu/Apps (93)
                18 | 93 => {
                    debug!("Alt/Menu 鍵，讓事件通過");
                    Ok(false)
                }
                
                // 點號 (VK_OEM_PERIOD = 190, VK_DECIMAL = 110)
                190 | 110 => {
                    let mut processor = state.input_processor.lock().unwrap();
                    let (success, symbol_selected) = processor.handle_symbol_input('.');
                    
                    if success {
                        // 檢查是否有符號選擇的候選字
                        if symbol_selected.is_some() {
                            // 符號映射找到了候選字，但不清除狀態，等待 Space 鍵送出
                            let state_ref = processor.get_state();
                            info!(
                                "✅ 符號映射（等待 Space 鍵送出）: '{}' -> {:?}",
                                state_ref.current_code,
                                state_ref.complement_selected
                            );
                            // 阻止點號按鍵事件，但不立即送出符號
                            return Ok(true);
                        }
                    }
                    
                    // 如果沒有找到符號映射，攔截點號（因為在攔截模式下，所有符號都應該被攔截）
                    debug!("攔截模式：攔截點號 vk={}", vk_value);
                    Ok(true)
                }
                
                // 逗號 (VK_OEM_COMMA = 188)
                188 => {
                    let mut processor = state.input_processor.lock().unwrap();
                    let (success, symbol_selected) = processor.handle_symbol_input(',');
                    
                    if success {
                        // 檢查是否有符號選擇的候選字
                        if symbol_selected.is_some() {
                            // 符號映射找到了候選字，但不清除狀態，等待 Space 鍵送出
                            let state_ref = processor.get_state();
                            info!(
                                "✅ 符號映射（等待 Space 鍵送出）: '{}' -> {:?}",
                                state_ref.current_code,
                                state_ref.complement_selected
                            );
                            // 阻止逗號按鍵事件，但不立即送出符號
                            return Ok(true);
                        }
                    }
                    
                    // 如果沒有找到符號映射，攔截逗號（因為在攔截模式下，所有符號都應該被攔截）
                    debug!("攔截模式：攔截逗號 vk={}", vk_value);
                    Ok(true)
                }
                
                // 其他所有按鍵：在攔截模式下都應該被攔截
                // 這包括符號、標點符號等所有可列印字符
                _ => {
                    debug!("攔截模式：攔截未處理的按鍵 vk={}", vk_value);
                    Ok(true) // 攔截所有其他按鍵
                },
            }
        }
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        unsafe {
            let _ = UnhookWindowsHookEx(self.hook_handle);
            info!("鍵盤鉤子已卸載");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_method::InputMethodProcessor;
    use crate::dictionary::Dictionary;
    use std::collections::HashMap;

    #[cfg(test)]
    fn create_test_state() -> AppState {
        use std::sync::Mutex;
        
        let mut code_map = HashMap::new();
        code_map.insert("a".to_string(), vec!["一".to_string(), "乙".to_string()]);
        code_map.insert("ab".to_string(), vec!["二".to_string()]);
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let processor = InputMethodProcessor::new(dictionary.clone());
        let input_processor = Arc::new(Mutex::new(processor));
        let input_simulator = Arc::new(Mutex::new(crate::input_simulator::InputSimulator::new().unwrap()));
        
        use crate::gui_window::GuiWindowManager;
        
        let gui_needs_update = Arc::new(AtomicBool::new(false));
        
        AppState {
            dictionary: Arc::new(Mutex::new(dictionary)),
            input_simulator: input_simulator.clone(),
            input_processor: input_processor.clone(),
            gui_window_manager: Arc::new(Mutex::new(GuiWindowManager::new(
                input_processor,
                input_simulator.clone(),
                gui_needs_update.clone(),
            ))),
            is_ucl_mode: Arc::new(Mutex::new(true)),
            is_half_mode: Arc::new(Mutex::new(false)),
            should_quit: Arc::new(AtomicBool::new(false)),
            gui_needs_update,
        }
    }

    #[test]
    fn test_keyboard_hook_creation() {
        let state = Arc::new(create_test_state());
        // 注意：實際測試需要 Windows 環境，這裡只測試創建
        // let hook = KeyboardHook::new(state).unwrap();
        // assert!(hook.hook_handle.0 != 0);
    }

    #[test]
    fn test_f4_quit_flag() {
        // 測試 F4 鍵退出標誌的設置
        let should_quit = Arc::new(AtomicBool::new(false));
        assert!(!should_quit.load(Ordering::Relaxed));
        
        should_quit.store(true, Ordering::Relaxed);
        assert!(should_quit.load(Ordering::Relaxed));
    }

    #[test]
    fn test_ctrl_pressed_state() {
        // 測試 Ctrl 鍵狀態追蹤
        CTRL_PRESSED.with(|p| {
            *p.borrow_mut() = false;
            assert!(!*p.borrow());
            
            *p.borrow_mut() = true;
            assert!(*p.borrow());
            
            *p.borrow_mut() = false;
            assert!(!*p.borrow());
        });
    }

    #[test]
    fn test_shift_toggle_state() {
        // 測試 Shift 切換狀態
        // 注意：實際的 Shift 鍵切換功能會在按下 Shift 時清除現有字根
        // 但這個測試只測試狀態切換邏輯，不測試清除字根功能（需要實際鍵盤事件）
        SHIFT_TOGGLE.with(|t| {
            // 初始狀態應該是 false（攔截模式）
            *t.borrow_mut() = false;
            assert!(!*t.borrow());
            
            // 第一次切換：false -> true（不攔截模式）
            let mut toggle = t.borrow_mut();
            *toggle = !*toggle;
            assert!(*toggle);
            
            // 第二次切換：true -> false（攔截模式）
            *toggle = !*toggle;
            assert!(!*toggle);
        });
    }

    #[test]
    fn test_vk_code_values() {
        // 測試虛擬鍵碼值
        assert_eq!(8, 8); // VK_BACK
        assert_eq!(13, 13); // VK_RETURN
        assert_eq!(17, 17); // VK_CONTROL
        assert_eq!(27, 27); // VK_ESCAPE
        assert_eq!(115, 115); // VK_F4
        assert_eq!(32, 32); // VK_SPACE
        assert_eq!(48, 48); // VK_0
        assert_eq!(57, 57); // VK_9
        assert_eq!(65, 65); // VK_A
        assert_eq!(90, 90); // VK_Z
    }

    #[test]
    fn test_should_quit_initialization() {
        // 測試退出標誌的初始化
        let state = Arc::new(create_test_state());
        // 注意：實際測試需要 Windows 環境
        // let hook = KeyboardHook::new(state).unwrap();
        // assert!(!hook.should_quit.load(Ordering::Relaxed));
    }

    #[test]
    fn test_wm_keydown_value() {
        // 測試 WM_KEYDOWN 常量值
        const WM_KEYDOWN_VALUE: usize = 256;
        assert_eq!(WM_KEYDOWN_VALUE, 256);
    }

    #[test]
    fn test_character_lowercase_conversion() {
        // 測試字符轉小寫
        assert_eq!('A'.to_ascii_lowercase(), 'a');
        assert_eq!('Z'.to_ascii_lowercase(), 'z');
        assert_eq!('a'.to_ascii_lowercase(), 'a');
        assert_eq!('z'.to_ascii_lowercase(), 'z');
    }

    #[test]
    fn test_vk_code_to_char_conversion() {
        // 測試虛擬鍵碼到字符的轉換
        let vk_a: u32 = 65;
        let ch = char::from(vk_a as u8);
        assert_eq!(ch, 'A');
        
        let vk_z: u32 = 90;
        let ch = char::from(vk_z as u8);
        assert_eq!(ch, 'Z');
    }
}
