//! 系統托盤模組

use crate::AppState;
use anyhow::Result;
use log::info;
use std::sync::Arc;
use tray_icon::{
    menu::{Menu, MenuItem},
    TrayIconBuilder,
};

/// 系統托盤圖示
pub struct TrayIcon {
    _tray_icon: tray_icon::TrayIcon,
    _state: Arc<AppState>,
}

impl TrayIcon {
    pub fn new(state: Arc<AppState>) -> Result<Self> {
        // 載入圖示（暫時使用預設）
        // TODO: 載入實際的 icon.ico
        
        let menu = Menu::new();
        
        // 創建退出選項
        // tray-icon 0.10 使用 Windows 消息循環處理菜單項點擊
        // 退出選項會自動發送 WM_COMMAND 消息，我們在 keyboard_hook.rs 中處理
        // 注意：MenuItem::new 的第三個參數是 Accelerator（快捷鍵），不是回調函數
        let quit_i = MenuItem::new("退出", true, None);
        menu.append(&quit_i)?;
        
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("肥米輸入法")
            .build()?;
        
        info!("系統托盤圖示已創建");
        
        Ok(Self {
            _tray_icon: tray_icon,
            _state: state,
        })
    }
    
    /// 獲取托盤圖示的窗口句柄（用於調試）
    pub fn _get_hwnd(&self) -> Option<windows::Win32::Foundation::HWND> {
        // tray-icon 0.10 可能不直接暴露窗口句柄
        // 這裡暫時返回 None，如果需要可以通過其他方式獲取
        None
    }
}

