//! 鍵盤輸入模擬模組

use anyhow::Result;
use log::debug;
use std::time::Duration;
use std::thread;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

/// 輸入模擬器
pub struct InputSimulator {
    // 暫時不使用 enigo，改用 Windows API
}

impl InputSimulator {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
    
    /// 發送文字（使用剪貼簿貼上方式）
    pub fn send_text_paste(&mut self, text: &str) -> Result<()> {
        use arboard::Clipboard;
        
        debug!("發送文字（貼上模式）: {}", text);
        
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(text)?;
        
        // 等待剪貼簿更新
        thread::sleep(Duration::from_millis(10));
        
        // 發送 Ctrl+V (使用 Windows API)
        unsafe {
            // 按下 Ctrl
            let mut input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(VK_CONTROL.0),
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            
            // 按下 V
            input.Anonymous.ki.wVk = VIRTUAL_KEY(VK_V.0);
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            
            // 釋放 V
            input.Anonymous.ki.dwFlags = KEYBD_EVENT_FLAGS(KEYEVENTF_KEYUP.0);
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            
            // 釋放 Ctrl
            input.Anonymous.ki.wVk = VIRTUAL_KEY(VK_CONTROL.0);
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        
        Ok(())
    }
    
    /// 發送文字（直接輸入方式）
    /// TODO: 實作 Unicode 字元輸入
    pub fn send_text_direct(&mut self, text: &str) -> Result<()> {
        debug!("發送文字（直接輸入）: {}", text);
        
        // 暫時使用貼上模式
        // TODO: 實作真正的直接輸入
        self.send_text_paste(text)
    }
}

