//! 遊戲輸入測試工具（獨立可執行文件）
//! 
//! 使用方法：
//!   cargo run --bin game_input_test
//!
//! 這個工具會模擬遊戲輸入場景，測試輸入窗口功能

mod game_input_test {
    use enigo::*;
    use std::thread;
    use std::time::Duration;
    use log::{info, debug};

    /// 遊戲輸入測試器
    pub struct GameInputTester {
        enigo: Enigo,
    }

    impl GameInputTester {
        pub fn new() -> Self {
            Self {
                enigo: Enigo::new(),
            }
        }

        /// 模擬按鍵按下
        pub fn key_down(&mut self, key: Key) {
            debug!("模擬按鍵按下: {:?}", key);
            self.enigo.key_down(key);
            thread::sleep(Duration::from_millis(10));
        }

        /// 模擬按鍵釋放
        pub fn key_up(&mut self, key: Key) {
            debug!("模擬按鍵釋放: {:?}", key);
            self.enigo.key_up(key);
            thread::sleep(Duration::from_millis(10));
        }

        /// 模擬按鍵點擊（按下+釋放）
        pub fn key_click(&mut self, key: Key) {
            self.key_down(key);
            self.key_up(key);
        }

        /// 模擬組合鍵（例如 Ctrl+V）
        pub fn key_combination(&mut self, keys: &[Key]) {
            info!("模擬組合鍵: {:?}", keys);
            
            // 按下所有修飾鍵
            for key in keys.iter().take(keys.len() - 1) {
                self.key_down(*key);
            }
            
            // 按下最後一個鍵
            if let Some(last_key) = keys.last() {
                self.key_click(*last_key);
            }
            
            // 釋放所有修飾鍵（逆序）
            for key in keys.iter().rev().skip(1) {
                self.key_up(*key);
            }
        }

        /// 模擬輸入文字（逐字符輸入）
        pub fn type_text(&mut self, text: &str) {
            info!("模擬輸入文字: {}", text);
            for ch in text.chars() {
                // enigo 的 key 方法主要支持 ASCII 字符
                // 對於中文字符，可能需要使用其他方法
                if ch.is_ascii() {
                    // 轉換為 Key
                    if let Some(key) = char_to_key(ch) {
                        self.key_click(key);
                    } else {
                        // 使用剪貼簿方式輸入非 ASCII 字符
                        debug!("字符 '{}' 無法直接輸入，跳過", ch);
                    }
                } else {
                    debug!("非 ASCII 字符 '{}'，無法直接輸入", ch);
                }
                thread::sleep(Duration::from_millis(50));
            }
        }

        /// 模擬 Ctrl+Space 熱鍵（唯一熱鍵）
        pub fn trigger_input_window(&mut self) {
            info!("模擬 Ctrl+Space 熱鍵（觸發輸入窗口）");
            self.key_combination(&[Key::Control, Key::Space]);
        }

        /// 模擬在輸入窗口中輸入字根
        pub fn input_code(&mut self, code: &str) {
            info!("模擬在輸入窗口中輸入字根: {}", code);
            self.type_text(code);
        }

        /// 模擬選擇候選字（數字鍵 1-9）
        pub fn select_candidate(&mut self, index: usize) {
            if index >= 1 && index <= 9 {
                let key = match index {
                    1 => Key::Layout('1'),
                    2 => Key::Layout('2'),
                    3 => Key::Layout('3'),
                    4 => Key::Layout('4'),
                    5 => Key::Layout('5'),
                    6 => Key::Layout('6'),
                    7 => Key::Layout('7'),
                    8 => Key::Layout('8'),
                    9 => Key::Layout('9'),
                    _ => return,
                };
                info!("模擬選擇候選字 {} (按鍵 {})", index, index);
                self.key_click(key);
            }
        }

        /// 模擬 Space 鍵選擇第一個候選字
        pub fn select_first_candidate(&mut self) {
            info!("模擬 Space 鍵選擇第一個候選字");
            self.key_click(Key::Space);
        }

        /// 模擬 ESC 鍵關閉窗口
        pub fn close_window(&mut self) {
            info!("模擬 ESC 鍵關閉窗口");
            self.key_click(Key::Escape);
        }

        /// 模擬完整的輸入流程
        /// 1. 觸發輸入窗口
        /// 2. 輸入字根
        /// 3. 選擇候選字
        pub fn simulate_full_input_flow(&mut self, code: &str, candidate_index: Option<usize>) {
            info!("開始模擬完整的輸入流程");
            
            // 步驟 1: 觸發輸入窗口
            self.trigger_input_window();
            thread::sleep(Duration::from_millis(200));
            
            // 步驟 2: 輸入字根
            self.input_code(code);
            thread::sleep(Duration::from_millis(300));
            
            // 步驟 3: 選擇候選字
            if let Some(index) = candidate_index {
                self.select_candidate(index);
            } else {
                self.select_first_candidate();
            }
            thread::sleep(Duration::from_millis(200));
            
            info!("完整的輸入流程模擬完成");
        }
    }

    /// 將字符轉換為 enigo::Key
    fn char_to_key(ch: char) -> Option<Key> {
        match ch {
            'a'..='z' | 'A'..='Z' => Some(Key::Layout(ch.to_ascii_lowercase())),
            '0'..='9' => Some(Key::Layout(ch)),
            ' ' => Some(Key::Space),
            '\n' => Some(Key::Return),
            '\t' => Some(Key::Tab),
            _ => None,
        }
    }
}

use game_input_test::GameInputTester;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() {
    println!("遊戲輸入測試工具");
    println!("==================");
    println!();
    println!("注意：這個工具會實際發送按鍵到系統，請確保：");
    println!("1. 輸入法程序正在運行");
    println!("2. 準備好接收輸入的應用程序（例如記事本）");
    println!("3. 5 秒後開始測試...");
    println!();
    
    // 等待 5 秒，讓用戶準備
    for i in (1..=5).rev() {
        print!("\r{} 秒後開始...", i);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_secs(1));
    }
    println!("\r開始測試...     ");
    println!();
    
    let mut tester = GameInputTester::new();
    
    // 測試 1: 觸發輸入窗口
    println!("測試 1: 觸發輸入窗口 (Ctrl+Space)");
    tester.trigger_input_window();
    thread::sleep(Duration::from_millis(500));
    
    // 測試 2: 輸入字根
    println!("測試 2: 輸入字根 'ucl'");
    tester.input_code("ucl");
    thread::sleep(Duration::from_millis(500));
    
    // 測試 3: 選擇第一個候選字
    println!("測試 3: 選擇第一個候選字 (Space)");
    tester.select_first_candidate();
    thread::sleep(Duration::from_millis(500));
    
    println!();
    println!("測試完成！");
    println!();
    println!("如果輸入窗口正常顯示並輸入了候選字，測試成功。");
    println!("如果沒有，請檢查：");
    println!("1. 輸入法程序是否正在運行");
    println!("2. 日誌輸出是否有錯誤");
    println!("3. 輸入窗口是否正確顯示");
}

