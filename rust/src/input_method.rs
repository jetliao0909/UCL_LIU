//! 輸入法邏輯模組

use crate::dictionary::Dictionary;
use log::debug;

/// 輸入法狀態
#[derive(Debug, Clone, PartialEq)]
pub struct InputMethodState {
    /// 當前輸入的字根
    pub current_code: String,
    /// 候選字列表
    pub candidates: Vec<String>,
    /// 當前候選字索引（用於分頁）
    pub candidate_index: usize,
    /// 每頁顯示的候選字數量
    pub candidates_per_page: usize,
    /// 補碼選擇的候選字（等待 Space 鍵送出）
    pub complement_selected: Option<String>,
}

impl Default for InputMethodState {
    fn default() -> Self {
        Self {
            current_code: String::new(),
            candidates: Vec::new(),
            candidate_index: 0,
            candidates_per_page: 6,
            complement_selected: None,
        }
    }
}

impl InputMethodState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 清除當前輸入
    pub fn clear(&mut self) {
        self.current_code.clear();
        self.candidates.clear();
        self.candidate_index = 0;
        self.complement_selected = None;
    }

    /// 添加字根
    pub fn append_code(&mut self, ch: char) {
        // 字根最多 5 碼
        if self.current_code.len() < 5 {
            self.current_code.push(ch);
            // 每次添加字根時，清除之前的補碼/符號選擇（因為開始輸入新字根）
            self.complement_selected = None;
        }
    }

    /// 刪除最後一個字根
    pub fn delete_last_code(&mut self) {
        if !self.current_code.is_empty() {
            self.current_code.pop();
        }
    }

    /// 查詢候選字
    pub fn lookup_candidates(&mut self, dictionary: &Dictionary) {
        if self.current_code.is_empty() {
            self.candidates.clear();
            self.candidate_index = 0;
            return;
        }

        if let Some(chars) = dictionary.lookup(&self.current_code) {
            self.candidates = chars.clone();
            self.candidate_index = 0;
            debug!(
                "查詢字根 '{}' 找到 {} 個候選字",
                self.current_code,
                self.candidates.len()
            );
        } else {
            self.candidates.clear();
            self.candidate_index = 0;
            debug!("查詢字根 '{}' 未找到候選字", self.current_code);
        }
    }

    /// 取得當前頁的候選字
    pub fn get_current_page_candidates(&self) -> Vec<String> {
        let start = self.candidate_index;
        let end = (start + self.candidates_per_page).min(self.candidates.len());
        
        if start >= self.candidates.len() {
            return Vec::new();
        }
        
        self.candidates[start..end].to_vec()
    }

    /// 是否有下一頁
    pub fn has_next_page(&self) -> bool {
        self.candidate_index + self.candidates_per_page < self.candidates.len()
    }

    /// 是否有上一頁
    pub fn has_prev_page(&self) -> bool {
        self.candidate_index > 0
    }

    /// 切換到下一頁
    pub fn next_page(&mut self) {
        if self.has_next_page() {
            self.candidate_index += self.candidates_per_page;
        }
    }

    /// 切換到上一頁
    pub fn prev_page(&mut self) {
        if self.has_prev_page() {
            self.candidate_index = self.candidate_index.saturating_sub(self.candidates_per_page);
        }
    }

    /// 根據數字鍵選擇候選字（0-9）
    /// 返回選中的字，如果無效返回 None
    pub fn select_candidate(&self, index: usize) -> Option<String> {
        let page_candidates = self.get_current_page_candidates();
        if index < page_candidates.len() {
            Some(page_candidates[index].clone())
        } else {
            None
        }
    }
}

/// 輸入法處理器
pub struct InputMethodProcessor {
    state: InputMethodState,
    dictionary: Dictionary,
}

impl InputMethodProcessor {
    pub fn new(dictionary: Dictionary) -> Self {
        Self {
            state: InputMethodState::new(),
            dictionary,
        }
    }

    /// 處理字根輸入
    /// 返回 (是否處理成功, 補碼選擇的候選字)
    pub fn handle_code_input(&mut self, ch: char) -> (bool, Option<String>) {
        // 只接受 a-z 的字根
        if !ch.is_ascii_lowercase() && !ch.is_ascii_uppercase() {
            return (false, None);
        }

        let ch_lower = ch.to_ascii_lowercase();
        
        // 補碼機制：v/r/s/f/w 分別選擇候選2/3/4/5/6
        // 如果輸入的是 v/r/s/f/w，且當前字根（加上補碼後）不在字典中，
        // 但當前字根（不加補碼）存在，則選擇對應的候選字
        // 
        // 補碼機制的觸發條件（參考 Python 版本的實現）：
        // 1. 加上補碼後的字根不在字典中
        // 2. 當前字根不為空
        // 3. 當前字根存在且有足夠的候選字
        // 4. 如果加上補碼後的字根長度 < 5，檢查是否有以該組合開頭的更長字根
        //    如果沒有，則觸發補碼；如果有，則不觸發（讓用戶繼續輸入）
        // 5. 如果加上補碼後的字根長度 = 5，如果不在字典中，應該觸發補碼
        // 
        // 補碼對應關係（參考 Python 版本）：
        // - v: 候選2（索引1），需要 >= 2 個候選字
        // - r: 候選3（索引2），需要 >= 3 個候選字
        // - s: 候選4（索引3），需要 >= 4 個候選字
        // - f: 候選5（索引4），需要 >= 5 個候選字
        // - w: 候選6（索引5），需要 >= 6 個候選字
        if ch_lower == 'v' || ch_lower == 'r' || ch_lower == 's' || ch_lower == 'f' || ch_lower == 'w' {
            let current_code = self.state.current_code.clone();
            
            // 先嘗試加上補碼後的字根
            let code_with_suffix = format!("{}{}", current_code, ch_lower);
            let exists_with_suffix = self.dictionary.lookup(&code_with_suffix).is_some();
            
            if !exists_with_suffix && !current_code.is_empty() {
                // 檢查當前字根（不加補碼）是否存在
                if let Some(candidates) = self.dictionary.lookup(&current_code) {
                    // 根據補碼字符確定候選字索引和所需的最小候選字數量
                    let (candidate_index, min_candidates) = match ch_lower {
                        'v' => (1, 2), // v 選擇候選2（索引1），需要 >= 2 個候選字
                        'r' => (2, 3), // r 選擇候選3（索引2），需要 >= 3 個候選字
                        's' => (3, 4), // s 選擇候選4（索引3），需要 >= 4 個候選字
                        'f' => (4, 5), // f 選擇候選5（索引4），需要 >= 5 個候選字
                        'w' => (5, 6), // w 選擇候選6（索引5），需要 >= 6 個候選字
                        _ => return (false, None), // 不應該到達這裡
                    };
                    
                    // 檢查候選字數量是否足夠
                    if candidates.len() >= min_candidates && candidates.len() > candidate_index {
                        // 判斷是否應該觸發補碼
                        let should_trigger_complement = if code_with_suffix.len() < 5 {
                            // 長度 < 5，檢查是否有以 code_with_suffix 開頭的更長字根
                            // 例如："si" + "s" = "sis"（3碼），檢查是否有 "sisp" 等
                            // 如果沒有，則觸發補碼；如果有，則不觸發（讓用戶繼續輸入）
                            !self.dictionary.has_prefix(&code_with_suffix)
                        } else {
                            // 長度 = 5，已經達到最大長度，如果不在字典中，應該觸發補碼
                            // 因為無法繼續輸入更長的字根
                            true
                        };
                        
                        if should_trigger_complement {
                            // 選擇對應的候選字，存儲在狀態中等待 Space 鍵送出
                            let selected = candidates[candidate_index].clone();
                            self.state.complement_selected = Some(selected.clone());
                            // 不清除字根，保持當前狀態，等待 Space 鍵
                            return (true, Some(selected));
                        }
                    }
                }
            }
            
            // 如果補碼機制不適用，繼續正常流程（添加補碼字符作為字根）
            self.state.append_code(ch_lower);
            self.state.lookup_candidates(&self.dictionary);
            return (true, None);
        }
        
        // 正常添加字根
        self.state.append_code(ch_lower);
        self.state.lookup_candidates(&self.dictionary);
        (true, None)
    }

    /// 處理符號輸入（例如點號 `.`）
    /// 返回 (是否處理成功, 符號選擇的候選字)
    /// 
    /// 與 Python 版本一致：完全依賴字典表查找，不進行硬編碼處理
    /// 字典表中的映射：
    /// - "." → "。"
    /// - "," → "，"
    /// - ".." → "："
    /// - ".," → "；"
    /// 
    /// 處理邏輯：
    /// 1. 如果當前有字根，先查找 字根+符號 的組合（例如 "s." 對應 "？"，".." 對應 "："）
    /// 2. 如果沒有字根，先將符號添加到字根中，然後查找組合（例如 "." + "." = ".."）
    /// 3. 如果組合不存在，再查找單獨的符號（例如 "." 對應 "。"）
    pub fn handle_symbol_input(&mut self, symbol: char) -> (bool, Option<String>) {
        let current_code = self.state.current_code.clone();
        
        // 如果當前有字根，嘗試查找 字根+符號 的組合（例如 "s." 對應 "？"，".." 對應 "："）
        if !current_code.is_empty() {
            let code_with_symbol = format!("{}{}", current_code, symbol);
            
            // 查詢字典中是否有這個符號組合
            if let Some(candidates) = self.dictionary.lookup(&code_with_symbol) {
                if let Some(first_symbol) = candidates.first() {
                    // 找到符號映射，存儲在狀態中等待 Space 鍵送出
                    let selected = first_symbol.clone();
                    self.state.complement_selected = Some(selected.clone());
                    // 不清除字根，保持當前狀態，等待 Space 鍵
                    debug!("✅ 從字典表找到符號映射: '{}' -> '{}'", code_with_symbol, selected);
                    return (true, Some(selected));
                }
            }
        }
        
        // 如果沒有字根，先將符號添加到字根中，然後查找組合
        // 這樣可以支持連續輸入符號（例如 ".." -> "："）
        if current_code.is_empty() {
            self.state.append_code(symbol);
            let new_code = self.state.current_code.clone();
            
            // 查找組合（例如 "." + "." = ".."）
            if let Some(candidates) = self.dictionary.lookup(&new_code) {
                if let Some(first_symbol) = candidates.first() {
                    // 找到組合映射，存儲在狀態中等待 Space 鍵送出
                    let selected = first_symbol.clone();
                    self.state.complement_selected = Some(selected.clone());
                    debug!("✅ 從字典表找到符號組合映射: '{}' -> '{}'", new_code, selected);
                    return (true, Some(selected));
                }
            }
            
            // 如果組合不存在，查找單獨的符號（例如 "." 對應 "。"）
            let symbol_str = symbol.to_string();
            if let Some(candidates) = self.dictionary.lookup(&symbol_str) {
                if let Some(first_symbol) = candidates.first() {
                    // 找到單獨符號映射，存儲在狀態中等待 Space 鍵送出
                    let selected = first_symbol.clone();
                    self.state.complement_selected = Some(selected.clone());
                    // 字根已經包含符號，保持不變
                    debug!("✅ 從字典表找到單獨符號映射: '{}' -> '{}'", symbol_str, selected);
                    return (true, Some(selected));
                }
            }
            
            // 如果都沒有找到，移除剛才添加的符號
            self.state.current_code.pop();
            return (false, None);
        }
        
        // 如果沒有找到符號映射，不處理（讓事件通過）
        (false, None)
    }

    /// 處理數字鍵選擇候選字
    pub fn handle_number_selection(&mut self, num: u8) -> Option<String> {
        if num > 9 {
            return None;
        }

        // 數字鍵 0 對應索引 9（第 10 個候選字）
        let index = if num == 0 { 9 } else { (num - 1) as usize };
        
        if let Some(selected) = self.state.select_candidate(index) {
            let result = selected.clone();
            self.state.clear();
            Some(result)
        } else {
            None
        }
    }

    /// 處理 Backspace
    pub fn handle_backspace(&mut self) -> bool {
        if self.state.current_code.is_empty() {
            return false; // 沒有字根可刪除，讓事件通過
        }

        self.state.delete_last_code();
        self.state.lookup_candidates(&self.dictionary);
        true
    }

    /// 處理 Space（選擇第一個候選字或補碼選擇的候選字）
    pub fn handle_space(&mut self) -> Option<String> {
        // 優先檢查是否有補碼選擇的候選字
        if let Some(complement_selected) = self.state.complement_selected.take() {
            self.state.clear();
            return Some(complement_selected);
        }
        
        // 否則選擇第一個候選字
        if let Some(first) = self.state.candidates.first() {
            let result = first.clone();
            self.state.clear();
            Some(result)
        } else {
            None
        }
    }

    /// 處理 Enter（送出當前字根，不清除）
    pub fn handle_enter(&mut self) -> Option<String> {
        if !self.state.current_code.is_empty() {
            Some(self.state.current_code.clone())
        } else {
            None
        }
    }

    /// 取得當前狀態
    pub fn get_state(&self) -> &InputMethodState {
        &self.state
    }

    /// 清除狀態
    pub fn clear(&mut self) {
        self.state.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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

    #[test]
    fn test_append_code() {
        let mut state = InputMethodState::new();
        state.append_code('a');
        assert_eq!(state.current_code, "a");
        
        state.append_code('b');
        assert_eq!(state.current_code, "ab");
    }

    #[test]
    fn test_code_limit() {
        let mut state = InputMethodState::new();
        for _ in 0..6 {
            state.append_code('a');
        }
        assert_eq!(state.current_code.len(), 5); // 最多 5 碼
    }

    #[test]
    fn test_delete_last_code() {
        let mut state = InputMethodState::new();
        state.append_code('a');
        state.append_code('b');
        state.delete_last_code();
        assert_eq!(state.current_code, "a");
    }

    #[test]
    fn test_lookup_candidates() {
        let dictionary = create_test_dictionary();
        let mut state = InputMethodState::new();
        
        state.append_code('a');
        state.lookup_candidates(&dictionary);
        assert_eq!(state.candidates.len(), 2);
        assert_eq!(state.candidates[0], "一");
        assert_eq!(state.candidates[1], "乙");
    }

    #[test]
    fn test_get_current_page_candidates() {
        let dictionary = create_test_dictionary();
        let mut state = InputMethodState::new();
        
        // 創建一個有 10 個候選字的測試
        state.candidates = (0..10).map(|i| format!("候選{}", i)).collect();
        state.candidates_per_page = 6;
        
        let page1 = state.get_current_page_candidates();
        assert_eq!(page1.len(), 6);
        assert_eq!(page1[0], "候選0");
        
        state.next_page();
        let page2 = state.get_current_page_candidates();
        assert_eq!(page2.len(), 4);
        assert_eq!(page2[0], "候選6");
    }

    #[test]
    fn test_select_candidate() {
        let dictionary = create_test_dictionary();
        let mut state = InputMethodState::new();
        
        state.append_code('a');
        state.lookup_candidates(&dictionary);
        
        // 選擇第一個候選字（數字鍵 1）
        let selected = state.select_candidate(0);
        assert_eq!(selected, Some("一".to_string()));
        
        // 選擇第二個候選字（數字鍵 2）
        let selected = state.select_candidate(1);
        assert_eq!(selected, Some("乙".to_string()));
        
        // 選擇不存在的候選字
        let selected = state.select_candidate(2);
        assert_eq!(selected, None);
    }

    #[test]
    fn test_handle_code_input() {
        let dictionary = create_test_dictionary();
        let mut processor = InputMethodProcessor::new(dictionary);
        
        let (success, _) = processor.handle_code_input('a');
        assert!(success);
        assert_eq!(processor.get_state().current_code, "a");
        assert_eq!(processor.get_state().candidates.len(), 2);
        
        let (success, _) = processor.handle_code_input('b');
        assert!(success);
        assert_eq!(processor.get_state().current_code, "ab");
        assert_eq!(processor.get_state().candidates.len(), 1);
    }

    #[test]
    fn test_handle_number_selection() {
        let dictionary = create_test_dictionary();
        let mut processor = InputMethodProcessor::new(dictionary);
        
        let (_, _) = processor.handle_code_input('a');
        
        // 選擇第一個候選字（數字鍵 1）
        let selected = processor.handle_number_selection(1);
        assert_eq!(selected, Some("一".to_string()));
        assert_eq!(processor.get_state().current_code, ""); // 應該清除
        
        // 重新輸入
        let (_, _) = processor.handle_code_input('a');
        
        // 選擇第二個候選字（數字鍵 2）
        let selected = processor.handle_number_selection(2);
        assert_eq!(selected, Some("乙".to_string()));
    }

    #[test]
    fn test_handle_backspace() {
        let dictionary = create_test_dictionary();
        let mut processor = InputMethodProcessor::new(dictionary);
        
        let (_, _) = processor.handle_code_input('a');
        let (_, _) = processor.handle_code_input('b');
        assert_eq!(processor.get_state().current_code, "ab");
        
        assert!(processor.handle_backspace());
        assert_eq!(processor.get_state().current_code, "a");
        assert_eq!(processor.get_state().candidates.len(), 2); // 應該重新查詢
        
        assert!(processor.handle_backspace());
        assert_eq!(processor.get_state().current_code, "");
        
        // 空字根時應該返回 false，讓事件通過
        assert!(!processor.handle_backspace());
    }

    #[test]
    fn test_handle_space() {
        let dictionary = create_test_dictionary();
        let mut processor = InputMethodProcessor::new(dictionary);
        
        let (_, _) = processor.handle_code_input('a');
        
        let selected = processor.handle_space();
        assert_eq!(selected, Some("一".to_string()));
        assert_eq!(processor.get_state().current_code, ""); // 應該清除
        
        // 沒有候選字時
        let (_, _) = processor.handle_code_input('x'); // 不存在的字根
        let selected = processor.handle_space();
        assert_eq!(selected, None);
    }

    #[test]
    fn test_handle_enter() {
        let dictionary = create_test_dictionary();
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 注意：handle_enter() 的實現保持不變（返回字根），但實際使用時 Enter 鍵會調用 handle_space()
        // 在鍵盤鉤子中，Enter 鍵的行為與 Space 鍵一致：選擇第一個候選字並清除輸入
        // 這裡測試 handle_enter() 的原始行為（僅返回字根，不清除）
        let (_, _) = processor.handle_code_input('a');
        let (_, _) = processor.handle_code_input('b');
        
        let result = processor.handle_enter();
        assert_eq!(result, Some("ab".to_string()));
        // handle_enter() 不會清除字根，只是返回字根
        assert_eq!(processor.get_state().current_code, "ab");
        
        // 手動清除後，Enter 應該返回 None
        processor.clear();
        let result = processor.handle_enter();
        assert_eq!(result, None);
    }

    #[test]
    fn test_candidate_pagination() {
        let mut code_map = HashMap::new();
        // 創建一個有很多候選字的字根
        code_map.insert("test".to_string(), (1..=20).map(|i| format!("候選{}", i)).collect());
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let mut processor = InputMethodProcessor::new(dictionary);
        let (_, _) = processor.handle_code_input('t');
        let (_, _) = processor.handle_code_input('e');
        let (_, _) = processor.handle_code_input('s');
        let (_, _) = processor.handle_code_input('t');
        
        let state = processor.get_state();
        assert_eq!(state.candidates.len(), 20);
        assert_eq!(state.candidate_index, 0);
        
        // 測試分頁
        let page1 = state.get_current_page_candidates();
        assert_eq!(page1.len(), 6); // 每頁 6 個候選字
        
        // 測試候選字索引
        assert_eq!(state.candidate_index, 0);
    }

    #[test]
    fn test_multiple_code_inputs() {
        let dictionary = create_test_dictionary();
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 測試多個字根輸入
        let (success, _) = processor.handle_code_input('a');
        assert!(success);
        assert_eq!(processor.get_state().current_code, "a");
        
        let (success, _) = processor.handle_code_input('b');
        assert!(success);
        assert_eq!(processor.get_state().current_code, "ab");
        
        let (success, _) = processor.handle_code_input('c');
        assert!(success);
        assert_eq!(processor.get_state().current_code, "abc");
    }

    #[test]
    fn test_code_limit_processor() {
        let dictionary = create_test_dictionary();
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 測試字根長度限制（最多 5 碼）
        let (_, _) = processor.handle_code_input('a');
        let (_, _) = processor.handle_code_input('b');
        let (_, _) = processor.handle_code_input('c');
        let (_, _) = processor.handle_code_input('d');
        let (_, _) = processor.handle_code_input('e');
        
        assert_eq!(processor.get_state().current_code.len(), 5);
        
        // 嘗試輸入第 6 個字符，應該不會被接受
        let state_before = processor.get_state().current_code.clone();
        let (_, _) = processor.handle_code_input('f');
        assert_eq!(processor.get_state().current_code, state_before);
    }

    #[test]
    fn test_empty_candidate_handling() {
        let dictionary = create_test_dictionary();
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入不存在的字根
        let (_, _) = processor.handle_code_input('x');
        let (_, _) = processor.handle_code_input('y');
        let (_, _) = processor.handle_code_input('z');
        
        let state = processor.get_state();
        assert_eq!(state.current_code, "xyz");
        assert_eq!(state.candidates.len(), 0);
        
        // Space 應該返回 None
        let result = processor.handle_space();
        assert_eq!(result, None);
        
        // Enter 應該返回字根
        let result = processor.handle_enter();
        assert_eq!(result, Some("xyz".to_string()));
    }

    #[test]
    fn test_complement_code_v() {
        let dictionary = create_test_dictionary();
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入 'a'，有 2 個候選字：["一", "乙"]
        let (_, _) = processor.handle_code_input('a');
        assert_eq!(processor.get_state().current_code, "a");
        assert_eq!(processor.get_state().candidates.len(), 2);
        
        // 輸入 'v'，應該選擇候選2（索引1，即"乙"），但不清除狀態，等待 Space 鍵
        let (success, selected) = processor.handle_code_input('v');
        assert!(success);
        assert_eq!(selected, Some("乙".to_string()));
        assert_eq!(processor.get_state().current_code, "a"); // 不清除字根
        assert_eq!(processor.get_state().complement_selected, Some("乙".to_string())); // 存儲補碼選擇
        
        // 按 Space 鍵，應該送出補碼選擇的候選字
        let space_result = processor.handle_space();
        assert_eq!(space_result, Some("乙".to_string()));
        assert_eq!(processor.get_state().current_code, ""); // 現在才清除
        assert_eq!(processor.get_state().complement_selected, None); // 補碼選擇已清除
    }

    #[test]
    fn test_complement_code_s() {
        let mut code_map = HashMap::new();
        // 創建一個有至少 4 個候選字的字根（s 需要 >= 4 個候選字）
        code_map.insert("test".to_string(), vec!["候選1".to_string(), "候選2".to_string(), "候選3".to_string(), "候選4".to_string()]);
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入 'test'
        let (_, _) = processor.handle_code_input('t');
        let (_, _) = processor.handle_code_input('e');
        let (_, _) = processor.handle_code_input('s');
        let (_, _) = processor.handle_code_input('t');
        assert_eq!(processor.get_state().current_code, "test");
        
        // 輸入 's'，應該選擇候選4（索引3），但不清除狀態，等待 Space 鍵
        let (success, selected) = processor.handle_code_input('s');
        assert!(success);
        assert_eq!(selected, Some("候選4".to_string()));
        assert_eq!(processor.get_state().current_code, "test"); // 不清除字根
        assert_eq!(processor.get_state().complement_selected, Some("候選4".to_string())); // 存儲補碼選擇
        
        // 按 Space 鍵，應該送出補碼選擇的候選字
        let space_result = processor.handle_space();
        assert_eq!(space_result, Some("候選4".to_string()));
        assert_eq!(processor.get_state().current_code, ""); // 現在才清除
        assert_eq!(processor.get_state().complement_selected, None); // 補碼選擇已清除
    }

    #[test]
    fn test_complement_code_r() {
        let mut code_map = HashMap::new();
        // 創建一個有至少 3 個候選字的字根（r 需要 >= 3 個候選字）
        code_map.insert("test".to_string(), vec!["候選1".to_string(), "候選2".to_string(), "候選3".to_string()]);
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入 'test'
        let (_, _) = processor.handle_code_input('t');
        let (_, _) = processor.handle_code_input('e');
        let (_, _) = processor.handle_code_input('s');
        let (_, _) = processor.handle_code_input('t');
        assert_eq!(processor.get_state().current_code, "test");
        
        // 輸入 'r'，應該選擇候選3（索引2），但不清除狀態，等待 Space 鍵
        let (success, selected) = processor.handle_code_input('r');
        assert!(success);
        assert_eq!(selected, Some("候選3".to_string()));
        assert_eq!(processor.get_state().current_code, "test"); // 不清除字根
        assert_eq!(processor.get_state().complement_selected, Some("候選3".to_string())); // 存儲補碼選擇
        
        // 按 Space 鍵，應該送出補碼選擇的候選字
        let space_result = processor.handle_space();
        assert_eq!(space_result, Some("候選3".to_string()));
        assert_eq!(processor.get_state().current_code, ""); // 現在才清除
        assert_eq!(processor.get_state().complement_selected, None); // 補碼選擇已清除
    }

    #[test]
    fn test_complement_code_f() {
        let mut code_map = HashMap::new();
        // 創建一個有至少 5 個候選字的字根（f 需要 >= 5 個候選字）
        code_map.insert("test".to_string(), vec!["候選1".to_string(), "候選2".to_string(), "候選3".to_string(), "候選4".to_string(), "候選5".to_string()]);
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入 'test'
        let (_, _) = processor.handle_code_input('t');
        let (_, _) = processor.handle_code_input('e');
        let (_, _) = processor.handle_code_input('s');
        let (_, _) = processor.handle_code_input('t');
        assert_eq!(processor.get_state().current_code, "test");
        
        // 輸入 'f'，應該選擇候選5（索引4），但不清除狀態，等待 Space 鍵
        let (success, selected) = processor.handle_code_input('f');
        assert!(success);
        assert_eq!(selected, Some("候選5".to_string()));
        assert_eq!(processor.get_state().current_code, "test"); // 不清除字根
        assert_eq!(processor.get_state().complement_selected, Some("候選5".to_string())); // 存儲補碼選擇
        
        // 按 Space 鍵，應該送出補碼選擇的候選字
        let space_result = processor.handle_space();
        assert_eq!(space_result, Some("候選5".to_string()));
        assert_eq!(processor.get_state().current_code, ""); // 現在才清除
        assert_eq!(processor.get_state().complement_selected, None); // 補碼選擇已清除
    }

    #[test]
    fn test_complement_code_w() {
        let mut code_map = HashMap::new();
        // 創建一個有至少 6 個候選字的字根（w 需要 >= 6 個候選字）
        code_map.insert("test".to_string(), vec!["候選1".to_string(), "候選2".to_string(), "候選3".to_string(), "候選4".to_string(), "候選5".to_string(), "候選6".to_string()]);
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入 'test'
        let (_, _) = processor.handle_code_input('t');
        let (_, _) = processor.handle_code_input('e');
        let (_, _) = processor.handle_code_input('s');
        let (_, _) = processor.handle_code_input('t');
        assert_eq!(processor.get_state().current_code, "test");
        
        // 輸入 'w'，應該選擇候選6（索引5），但不清除狀態，等待 Space 鍵
        let (success, selected) = processor.handle_code_input('w');
        assert!(success);
        assert_eq!(selected, Some("候選6".to_string()));
        assert_eq!(processor.get_state().current_code, "test"); // 不清除字根
        assert_eq!(processor.get_state().complement_selected, Some("候選6".to_string())); // 存儲補碼選擇
        
        // 按 Space 鍵，應該送出補碼選擇的候選字
        let space_result = processor.handle_space();
        assert_eq!(space_result, Some("候選6".to_string()));
        assert_eq!(processor.get_state().current_code, ""); // 現在才清除
        assert_eq!(processor.get_state().complement_selected, None); // 補碼選擇已清除
    }

    #[test]
    fn test_symbol_input() {
        let mut code_map = HashMap::new();
        code_map.insert("s.".to_string(), vec!["？".to_string()]);
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入 's'
        let (_, _) = processor.handle_code_input('s');
        assert_eq!(processor.get_state().current_code, "s");
        
        // 輸入 '.'，應該找到符號映射 "s." -> "？"
        let (success, symbol_selected) = processor.handle_symbol_input('.');
        assert!(success);
        assert_eq!(symbol_selected, Some("？".to_string()));
        assert_eq!(processor.get_state().current_code, "s"); // 不清除字根
        assert_eq!(processor.get_state().complement_selected, Some("？".to_string())); // 存儲符號選擇
        
        // 按 Space 鍵，應該送出符號選擇的候選字
        let space_result = processor.handle_space();
        assert_eq!(space_result, Some("？".to_string()));
        assert_eq!(processor.get_state().current_code, ""); // 現在才清除
        assert_eq!(processor.get_state().complement_selected, None); // 符號選擇已清除
    }

    #[test]
    fn test_symbol_input_not_found() {
        let mut code_map = HashMap::new();
        code_map.insert("s".to_string(), vec!["一".to_string()]);
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入 's'
        let (_, _) = processor.handle_code_input('s');
        assert_eq!(processor.get_state().current_code, "s");
        
        // 輸入 '.'，但 "s." 不在字典中，應該不處理
        let (success, symbol_selected) = processor.handle_symbol_input('.');
        assert!(!success);
        assert_eq!(symbol_selected, None);
        assert_eq!(processor.get_state().current_code, "s"); // 字根保持不變
    }

    #[test]
    fn test_double_dot_to_colon() {
        // 測試 ".." 從字典表中查找對應 "："（全形冒號）
        let mut code_map = HashMap::new();
        code_map.insert(".".to_string(), vec!["。".to_string()]);
        code_map.insert("..".to_string(), vec!["：".to_string()]);
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入第一個點號，應該先添加到字根，然後查找單獨的 "." -> "。"
        let (success1, symbol1) = processor.handle_symbol_input('.');
        assert!(success1);
        assert_eq!(symbol1, Some("。".to_string()));
        // 字根應該包含點號（因為先添加了）
        assert_eq!(processor.state.current_code, ".");
        
        // 輸入第二個點號，字根已經是 "."，應該從字典表中找到 ".." -> "："
        let (success, symbol_selected) = processor.handle_symbol_input('.');
        assert!(success);
        assert_eq!(symbol_selected, Some("：".to_string()));
        assert_eq!(processor.state.complement_selected, Some("：".to_string()));
        // 字根保持不變（等待 Space 鍵送出）
        assert_eq!(processor.state.current_code, ".");
    }
    
    #[test]
    fn test_dot_comma_to_semicolon() {
        // 測試 ".," 從字典表中查找對應 "；"（全形分號）
        let mut code_map = HashMap::new();
        code_map.insert(".".to_string(), vec!["。".to_string()]);
        code_map.insert(",".to_string(), vec!["，".to_string()]);
        code_map.insert(".,".to_string(), vec!["；".to_string()]);
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入第一個點號，應該先添加到字根，然後查找單獨的 "." -> "。"
        let (success1, symbol1) = processor.handle_symbol_input('.');
        assert!(success1);
        assert_eq!(symbol1, Some("。".to_string()));
        // 字根應該包含點號（因為先添加了）
        assert_eq!(processor.state.current_code, ".");
        
        // 輸入逗號，字根已經是 "."，應該從字典表中找到 ".," -> "；"
        let (success, symbol_selected) = processor.handle_symbol_input(',');
        assert!(success);
        assert_eq!(symbol_selected, Some("；".to_string()));
        assert_eq!(processor.state.complement_selected, Some("；".to_string()));
        // 字根保持不變（等待 Space 鍵送出）
        assert_eq!(processor.state.current_code, ".");
    }
    
    fn test_symbol_input_standalone() {
        let mut code_map = HashMap::new();
        code_map.insert(".".to_string(), vec!["。".to_string()]);
        code_map.insert(",".to_string(), vec!["，".to_string()]);
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 測試單獨輸入 '.'，應該找到符號映射 "." -> "。"
        let (success, symbol_selected) = processor.handle_symbol_input('.');
        assert!(success);
        assert_eq!(symbol_selected, Some("。".to_string()));
        assert_eq!(processor.get_state().current_code, ""); // 沒有字根
        assert_eq!(processor.get_state().complement_selected, Some("。".to_string())); // 存儲符號選擇
        
        // 按 Space 鍵，應該送出符號選擇的候選字
        let space_result = processor.handle_space();
        assert_eq!(space_result, Some("。".to_string()));
        assert_eq!(processor.get_state().current_code, ""); // 保持為空
        assert_eq!(processor.get_state().complement_selected, None); // 符號選擇已清除
        
        // 測試單獨輸入 ','，應該找到符號映射 "," -> "，"
        let (success2, symbol_selected2) = processor.handle_symbol_input(',');
        assert!(success2);
        assert_eq!(symbol_selected2, Some("，".to_string()));
        assert_eq!(processor.get_state().complement_selected, Some("，".to_string())); // 存儲符號選擇
        
        // 按 Space 鍵，應該送出符號選擇的候選字
        let space_result2 = processor.handle_space();
        assert_eq!(space_result2, Some("，".to_string()));
        assert_eq!(processor.get_state().complement_selected, None); // 符號選擇已清除
    }

    #[test]
    fn test_complement_code_v_not_applicable() {
        let mut code_map = HashMap::new();
        code_map.insert("av".to_string(), vec!["測試".to_string()]);
        code_map.insert("a".to_string(), vec!["一".to_string(), "乙".to_string()]);
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let mut processor = InputMethodProcessor::new(dictionary);
        let (_, _) = processor.handle_code_input('a');
        
        // 輸入 'v'，因為 "av" 在字典中，應該正常添加 'v' 作為字根
        let (success, selected) = processor.handle_code_input('v');
        assert!(success);
        assert_eq!(selected, None); // 不應該選擇候選字
        assert_eq!(processor.get_state().current_code, "av"); // 應該添加 'v'
    }

    #[test]
    fn test_complement_code_hjv() {
        // 測試 "hjv" 應該觸發補碼
        // "hj" + "v" = "hjv"（長度 3 < 5），且沒有以 "hjv" 開頭的字根，應該觸發補碼
        let mut code_map = HashMap::new();
        code_map.insert("hj".to_string(), vec!["候選1".to_string(), "候選2".to_string()]);
        // 不添加 "hjv" 或任何以 "hjv" 開頭的字根
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入 'h'
        let (_, _) = processor.handle_code_input('h');
        // 輸入 'j'
        let (_, _) = processor.handle_code_input('j');
        assert_eq!(processor.get_state().current_code, "hj");
        assert_eq!(processor.get_state().candidates.len(), 2);
        
        // 輸入 'v'，應該選擇候選2（索引1），觸發補碼
        let (success, selected) = processor.handle_code_input('v');
        assert!(success);
        assert_eq!(selected, Some("候選2".to_string()));
        assert_eq!(processor.get_state().current_code, "hj"); // 不清除字根
        assert_eq!(processor.get_state().complement_selected, Some("候選2".to_string())); // 存儲補碼選擇
        
        // 按 Space 鍵，應該送出補碼選擇的候選字
        let space_result = processor.handle_space();
        assert_eq!(space_result, Some("候選2".to_string()));
        assert_eq!(processor.get_state().current_code, ""); // 現在才清除
        assert_eq!(processor.get_state().complement_selected, None); // 補碼選擇已清除
    }

    #[test]
    fn test_complement_code_sisp_not_triggered() {
        // 測試 "sisp" 不應該觸發補碼
        // "si" + "s" = "sis"（長度 3 < 5），但有 "sisp" 以 "sis" 開頭，所以不應該觸發補碼
        let mut code_map = HashMap::new();
        code_map.insert("si".to_string(), vec!["候選1".to_string(), "候選2".to_string(), "候選3".to_string()]);
        code_map.insert("sisp".to_string(), vec!["目標字".to_string()]); // 有 "sisp" 以 "sis" 開頭
        
        let dictionary = Dictionary {
            code_to_chars: code_map,
            pinyi_data: None,
        };
        
        let mut processor = InputMethodProcessor::new(dictionary);
        
        // 輸入 's'
        let (_, _) = processor.handle_code_input('s');
        // 輸入 'i'
        let (_, _) = processor.handle_code_input('i');
        assert_eq!(processor.get_state().current_code, "si");
        assert_eq!(processor.get_state().candidates.len(), 3);
        
        // 輸入 's'，不應該觸發補碼，應該正常添加 's' 作為字根
        let (success, selected) = processor.handle_code_input('s');
        assert!(success);
        assert_eq!(selected, None); // 不應該有補碼選擇
        assert_eq!(processor.get_state().current_code, "sis"); // 應該正常添加 's'
        assert_eq!(processor.get_state().complement_selected, None); // 不應該有補碼選擇
        
        // 繼續輸入 'p'，應該能找到 "sisp"
        let (success2, _) = processor.handle_code_input('p');
        assert!(success2);
        assert_eq!(processor.get_state().current_code, "sisp");
        // 應該找到 "sisp" 的候選字
        assert_eq!(processor.get_state().candidates.len(), 1);
        assert_eq!(processor.get_state().candidates[0], "目標字");
    }
}

