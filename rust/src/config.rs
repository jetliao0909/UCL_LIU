//! 配置管理模組

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 應用程式配置
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// 是否為「短」版模式
    pub short_mode: bool,
    /// 縮放大小
    pub zoom: f64,
    /// 透明度
    pub alpha: f64,
    /// 視窗位置 X
    pub x: i32,
    /// 視窗位置 Y
    pub y: i32,
    /// 是否顯示短根
    pub sp: bool,
    /// 是否有打字音
    pub play_sound_enable: bool,
    /// 啟動時預設模式（0=英模式，1=肥模式）
    pub startup_default_ucl: bool,
    /// 允許使用 Shift+Space 切換全形/半形
    pub enable_half_full: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            short_mode: false,
            zoom: 0.90,
            alpha: 1.0,
            x: 1239,
            y: 950,
            sp: false,
            play_sound_enable: false,
            startup_default_ucl: true,
            enable_half_full: true,
        }
    }
}

impl Config {
    /// 載入配置檔案
    pub fn load() -> Result<Self> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "無法取得執行檔目錄"
            ))?;
        
        let config_path = exe_dir.join("UCLLIU.ini");
        
        if !config_path.exists() {
            // 如果配置檔案不存在，使用預設值並創建檔案
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        
        // TODO: 解析 INI 格式（目前先使用預設值）
        Ok(Self::default())
    }
    
    /// 儲存配置檔案
    pub fn save(&self) -> Result<()> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "無法取得執行檔目錄"
            ))?;
        
        let _config_path = exe_dir.join("UCLLIU.ini");
        
        // TODO: 寫入 INI 格式
        // 目前先不實作
        
        Ok(())
    }
}

