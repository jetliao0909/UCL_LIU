//! 肥米輸入法 - Rust 版本 MVP
//! 
//! 核心功能：
//! 1. Windows 全域鍵盤鉤子
//! 2. 字碼表查詢
//! 3. 鍵盤輸入模擬
//! 4. 系統托盤圖示

mod keyboard_hook;
mod dictionary;
mod input_simulator;
mod input_method;
mod tray;
mod config;
mod gui_window;
mod game_input_test;

use anyhow::Result;
use log::{info, error, debug};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use dictionary::Dictionary;
use keyboard_hook::KeyboardHook;
use input_simulator::InputSimulator;
use input_method::InputMethodProcessor;
use tray::TrayIcon;
use gui_window::GuiWindowManager;

/// 應用程式狀態
pub struct AppState {
    dictionary: Arc<Mutex<Dictionary>>,
    input_simulator: Arc<Mutex<InputSimulator>>,
    input_processor: Arc<Mutex<InputMethodProcessor>>,
    gui_window_manager: Arc<Mutex<GuiWindowManager>>,
    is_ucl_mode: Arc<Mutex<bool>>,  // 肥/英模式
    is_half_mode: Arc<Mutex<bool>>, // 半/全模式
    should_quit: Arc<AtomicBool>,   // 退出標誌
    gui_needs_update: Arc<AtomicBool>, // GUI 需要更新標誌
}

impl AppState {
    fn new() -> Result<Self> {
        let dictionary = Arc::new(Mutex::new(Dictionary::load()?));
        let input_simulator = Arc::new(Mutex::new(InputSimulator::new()?));
        
        // 創建輸入法處理器
        let dict_for_processor = dictionary.lock().unwrap();
        let processor = InputMethodProcessor::new((*dict_for_processor).clone());
        drop(dict_for_processor);
        
        let input_processor = Arc::new(Mutex::new(processor));
        
        // 創建 GUI 需要更新標誌
        let gui_needs_update = Arc::new(AtomicBool::new(false));
        
        // 創建 GUI 窗口管理器
        let gui_window_manager = Arc::new(Mutex::new(GuiWindowManager::new(
            input_processor.clone(),
            input_simulator.clone(),
            gui_needs_update.clone(),
        )));
        
        Ok(Self {
            dictionary,
            input_simulator,
            input_processor,
            gui_window_manager,
            is_ucl_mode: Arc::new(Mutex::new(true)),
            is_half_mode: Arc::new(Mutex::new(false)),
            should_quit: Arc::new(AtomicBool::new(false)),
            gui_needs_update,
        })
    }
}

fn main() -> Result<()> {
    // 初始化日誌（使用 debug 級別以便看到鍵盤事件）
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    
    info!("肥米輸入法 Rust 版本啟動中...");
    
    // 檢查是否已有實例運行
    if !is_single_instance() {
        error!("肥米輸入法已在運行中");
        return Err(anyhow::anyhow!("已有實例運行"));
    }
    
    // 載入配置
    let _config = config::Config::load()?;
    
    // 初始化應用狀態
    let state = Arc::new(AppState::new()?);
    
    // 初始化 fltk
    let app = fltk::app::App::default();
    
    // 設置鍵盤鉤子（需要先設置，因為它會將 should_quit 存儲到 thread_local）
    let hook = KeyboardHook::new(state.clone())?;
    
    // 創建系統托盤（需要 should_quit 引用）
    let _tray = TrayIcon::new(state.clone())?;
    
    // 顯示 GUI 主窗口
    {
        let mut gui_manager = state.gui_window_manager.lock().unwrap();
        gui_manager.show()?;
    }
    
    info!("肥米輸入法已啟動，等待輸入...");
    info!("按 Ctrl+Space 打開/關閉右下角 GUI 狀態列（遊戲模式）");
    
    // 運行訊息循環（同時處理鍵盤事件、系統托盤事件和 fltk 事件）
    let result = hook.run_with_fltk(&app, state.clone());
    
    // 程序退出時清理鎖定文件（鎖已自動釋放，但文件會殘留）
    cleanup_lock_file();
    
    result
}

/// 清理鎖定文件
/// 注意：文件鎖在文件句柄被 drop 時已自動釋放
/// 這裡只是刪除殘留的文件本身
fn cleanup_lock_file() {
    use std::fs;
    
    if let Err(e) = fs::remove_file("UCLLIU.lock") {
        // 文件可能已被刪除或不存在，忽略錯誤
        debug!("清理鎖定文件時發生錯誤（可忽略）：{}", e);
    } else {
        info!("已清理鎖定文件");
    }
}

/// 檢查是否為單一實例
/// 使用文件鎖定機制防止重複執行
/// 當程序退出時，文件鎖會自動釋放（文件句柄被 drop）
fn is_single_instance() -> bool {
    use std::sync::Mutex;
    use std::fs::OpenOptions;
    use fs2::FileExt;
    
    static LOCK: Mutex<Option<std::fs::File>> = Mutex::new(None);
    
    let mut lock = LOCK.lock().unwrap();
    if lock.is_some() {
        // 已經有鎖了，不應該到達這裡
        return false;
    }
    
    // 嘗試創建鎖定檔案
    match OpenOptions::new()
        .create(true)
        .write(true)
        .open("UCLLIU.lock")
    {
        Ok(file) => {
            // 嘗試獲取獨占鎖（非阻塞）
            // 如果文件已被其他進程鎖定，會返回錯誤
            match file.try_lock_exclusive() {
                Ok(_) => {
                    // 成功獲取鎖，保存文件句柄
                    // 文件句柄會一直保持鎖定狀態，直到程序退出或文件被 drop
                    *lock = Some(file);
                    info!("成功獲取單一實例鎖");
                    true
                }
                Err(e) => {
                    // 鎖已被其他進程持有
                    error!("無法獲取單一實例鎖：{}（可能已有實例在運行）", e);
                    false
                }
            }
        }
        Err(e) => {
            error!("無法創建鎖定檔案：{}", e);
            false
        }
    }
}

