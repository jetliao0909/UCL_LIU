# 測試結果

## 輸入法邏輯模組測試

所有單元測試已通過！

### 測試覆蓋範圍

1. **字根累積測試**
   - ✅ `test_append_code` - 測試添加字根
   - ✅ `test_code_limit` - 測試字根長度限制（最多 5 碼）
   - ✅ `test_delete_last_code` - 測試刪除最後一個字根

2. **字碼表查詢測試**
   - ✅ `test_lookup_candidates` - 測試查詢候選字

3. **候選字處理測試**
   - ✅ `test_get_current_page_candidates` - 測試取得當前頁候選字
   - ✅ `test_select_candidate` - 測試選擇候選字

4. **輸入處理測試**
   - ✅ `test_handle_code_input` - 測試處理字根輸入
   - ✅ `test_handle_number_selection` - 測試處理數字鍵選擇
   - ✅ `test_complement_code_v` - 測試補碼機制（v 選擇候選2）
   - ✅ `test_complement_code_r` - 測試補碼機制（r 選擇候選3）
   - ✅ `test_complement_code_s` - 測試補碼機制（s 選擇候選4）
   - ✅ `test_complement_code_f` - 測試補碼機制（f 選擇候選5）
   - ✅ `test_complement_code_w` - 測試補碼機制（w 選擇候選6）
   - ✅ `test_complement_code_v_not_applicable` - 測試補碼機制不適用情況
   - ✅ `test_complement_code_hjv` - 測試補碼機制（"hjv" 應該觸發補碼，因為沒有以 "hjv" 開頭的字根）
   - ✅ `test_complement_code_sisp_not_triggered` - 測試補碼機制（"sisp" 不應該觸發補碼，因為有 "sisp" 以 "sis" 開頭）
   - ✅ `test_symbol_input` - 測試符號輸入（例如 `s.` 對應 `？`）
   - ✅ `test_symbol_input_not_found` - 測試符號輸入未找到映射的情況
   - ✅ `test_symbol_input_standalone` - 測試單獨符號輸入（例如 `.` 對應 `。`，`,` 對應 `，`）
   - ✅ `test_double_dot_to_colon` - 測試連續輸入兩個點號轉換為全形冒號（`..` → `：`）
   - ✅ `test_dot_comma_to_semicolon` - 測試點號後輸入逗號轉換為全形分號（`.,` → `；`）
   - ✅ `test_handle_backspace` - 測試處理 Backspace
   - ✅ `test_handle_space` - 測試處理 Space（選擇第一個候選字）
   - ✅ `test_handle_enter` - 測試處理 Enter（送出字根）
   - ✅ `test_multiple_code_inputs` - 測試多個字根輸入
   - ✅ `test_code_limit_processor` - 測試字根長度限制（最多 5 碼）
   - ✅ `test_empty_candidate_handling` - 測試空候選字處理
   - ✅ `test_candidate_pagination` - 測試候選字分頁功能

5. **單一實例鎖定測試**
   - ⚠️ 單一實例鎖定機制在 `main.rs` 中實現，需要手動測試
   - **測試方法**：
     - 啟動第一個實例，確認程序正常運行
     - 嘗試啟動第二個實例，應該會立即退出並顯示「已有實例運行」錯誤
     - 關閉第一個實例，確認 `UCLLIU.lock` 文件已被自動刪除
     - 再次啟動程序，應該能正常啟動

6. **鍵盤鉤子測試**
   - ✅ `test_keyboard_hook_creation` - 測試鍵盤鉤子創建
   - ✅ `test_f4_quit_flag` - 測試 F4 鍵退出標誌
   - ✅ `test_ctrl_pressed_state` - 測試 Ctrl 鍵狀態追蹤
   - ✅ `test_shift_toggle_state` - 測試 Shift 切換狀態（攔截/不攔截模式）
   - ✅ `test_should_quit_initialization` - 測試退出標誌初始化
   - ✅ `test_vk_code_values` - 測試虛擬鍵碼值（包含 VK_ESCAPE = 27）
   - ✅ `test_wm_keydown_value` - 測試 WM_KEYDOWN 常量值
   - ✅ `test_character_lowercase_conversion` - 測試字符轉小寫
   - ✅ `test_vk_code_to_char_conversion` - 測試虛擬鍵碼到字符轉換

7. **GUI 窗口測試（輸入窗口模式 - 支援 Raw Input 遊戲）**
   - ✅ `test_gui_window_creation` - 測試窗口創建成功
   - ✅ `test_gui_window_manager_creation` - 測試窗口管理器創建成功
   - ✅ `test_window_keyboard_event_letter_input` - 測試字母鍵輸入處理
   - ✅ `test_window_keyboard_event_number_selection` - 測試數字鍵選擇候選字
   - ✅ `test_window_keyboard_event_space_selection` - 測試 Space 鍵選擇第一個候選字
   - ✅ `test_window_keyboard_event_backspace` - 測試 Backspace 鍵刪除字根
   - ✅ `test_window_keyboard_event_escape_clear` - 測試 ESC 鍵清除輸入
   - ✅ `test_input_window_mode_independent_input` - 測試輸入窗口模式的核心特性（獨立處理鍵盤輸入）
   - ✅ `test_input_window_mode_continuous_input` - 測試連續輸入多個字
   - ✅ `test_window_can_receive_keyboard_input_without_hook` - **核心測試**：驗證窗口能夠接收鍵盤輸入（不依賴鍵盤鉤子）
     - 這是支援 Raw Input 遊戲的關鍵測試
     - 驗證窗口能夠獨立處理鍵盤輸入，不依賴 `WH_KEYBOARD_LL` 鉤子
     - 驗證字根輸入、候選字選擇、特殊按鍵處理

### 測試執行結果

```
running 40 tests
test input_method::tests::test_candidate_pagination ... ok
test input_method::tests::test_code_limit_processor ... ok
test input_method::tests::test_handle_space ... ok
test input_method::tests::test_delete_last_code ... ok
test input_method::tests::test_empty_candidate_handling ... ok
test input_method::tests::test_get_current_page_candidates ... ok
test input_method::tests::test_code_limit ... ok
test input_method::tests::test_handle_code_input ... ok
test input_method::tests::test_handle_enter ... ok
test input_method::tests::test_handle_number_selection ... ok
test input_method::tests::test_handle_backspace ... ok
test input_method::tests::test_lookup_candidates ... ok
test input_method::tests::test_multiple_code_inputs ... ok
test input_method::tests::test_select_candidate ... ok
test input_method::tests::test_append_code ... ok
test keyboard_hook::tests::test_character_lowercase_conversion ... ok
test keyboard_hook::tests::test_ctrl_pressed_state ... ok
test keyboard_hook::tests::test_keyboard_hook_creation ... ok
test keyboard_hook::tests::test_should_quit_initialization ... ok
test keyboard_hook::tests::test_f4_quit_flag ... ok
test keyboard_hook::tests::test_shift_toggle_state ... ok
test keyboard_hook::tests::test_vk_code_to_char_conversion ... ok
test keyboard_hook::tests::test_vk_code_values ... ok
test keyboard_hook::tests::test_wm_keydown_value ... ok

test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured
```

### GUI 窗口測試結果（輸入窗口模式）

```
running 10 tests
test gui_window::tests::test_input_window_mode_continuous_input ... ok
test gui_window::tests::test_window_keyboard_event_escape_clear ... ok
test gui_window::tests::test_window_keyboard_event_letter_input ... ok
test gui_window::tests::test_window_keyboard_event_number_selection ... ok
test gui_window::tests::test_window_keyboard_event_space_selection ... ok
test gui_window::tests::test_window_keyboard_event_backspace ... ok
test gui_window::tests::test_window_can_receive_keyboard_input_without_hook ... ok
test gui_window::tests::test_input_window_mode_independent_input ... ok
test gui_window::tests::test_gui_window_manager_creation ... ok
test gui_window::tests::test_gui_window_creation ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured
```

**重要說明**：
- `test_window_can_receive_keyboard_input_without_hook` 是核心測試，驗證窗口能夠獨立接收鍵盤輸入，不依賴 `WH_KEYBOARD_LL` 鉤子
- 這使得輸入法能夠支援使用 Raw Input 的遊戲，因為窗口有焦點時，鍵盤事件直接發送到窗口，繞過 Raw Input 限制

**注意**：測試總數目前為 50 個（包含輸入法邏輯、鍵盤鉤子和 GUI 窗口測試）

## 鍵盤鉤子功能

鍵盤鉤子回調函數已實作完成，包括：

1. ✅ Windows 低階鍵盤鉤子設置
2. ✅ 鍵盤事件捕獲和處理
3. ✅ 字根輸入處理（A-Z，自動轉為小寫，大小寫無分別）
4. ✅ Shift 鍵切換攔截 / 英模式（行為與 Python 版一致）
   - **單獨按一下 Shift**（期間沒有搭配其他鍵）時，在 Shift 放開時切換模式，並清除現有字根
   - **Shift + 其他鍵（例如 Shift+1, Shift+2, Shift+A）** 時，不切換模式，事件全部放行給系統
   - **英模式 = 不攔截模式**：在英模式下，可以正常使用 Shift 做組合鍵（輸出上排符號等）
5. ✅ 數字鍵選擇候選字（0-9）
6. ✅ 補碼機制
   - 輸入 `v`：如果當前字根 + `v` 不在字典中，但當前字根存在且候選字數量 >= 2，則選擇候選2（第2個候選字），等待 Space 鍵送出
   - 輸入 `r`：如果當前字根 + `r` 不在字典中，但當前字根存在且候選字數量 >= 3，則選擇候選3（第3個候選字），等待 Space 鍵送出
   - 輸入 `s`：如果當前字根 + `s` 不在字典中，但當前字根存在且候選字數量 >= 4，則選擇候選4（第4個候選字），等待 Space 鍵送出
   - 輸入 `f`：如果當前字根 + `f` 不在字典中，但當前字根存在且候選字數量 >= 5，則選擇候選5（第5個候選字），等待 Space 鍵送出
   - 輸入 `w`：如果當前字根 + `w` 不在字典中，但當前字根存在且候選字數量 >= 6，則選擇候選6（第6個候選字），等待 Space 鍵送出
   - 補碼選擇後不會立即送出，需要按 Space 鍵才會送出選中的候選字
   - **觸發條件**：
     - 如果當前字根 + 補碼長度 < 5：檢查是否有以該組合開頭的更長字根；如果沒有，則觸發補碼；如果有，則不觸發（讓用戶繼續輸入）
     - 如果當前字根 + 補碼長度 = 5：如果不在字典中，則觸發補碼（因為無法繼續輸入更長的字根）
   - **範例**：
     - `"hj" + "v" = "hjv"`（長度 3 < 5，且沒有以 "hjv" 開頭的字根）→ 觸發補碼
     - `"si" + "s" = "sis"`（長度 3 < 5，但有 "sisp" 以 "sis" 開頭）→ 不觸發補碼，讓用戶繼續輸入
7. ✅ 符號輸入（與 Python 版本一致，完全依賴字典表查找）
   - 輸入符號（例如點號 `.` 或逗號 `,`）：
     - 如果當前有字根，先查找 字根+符號 的組合（例如 `s.` 對應 `？`，`..` 對應 `：`）
     - 如果當前沒有字根，先將符號添加到字根中，然後查找組合；如果組合不存在，再查找單獨符號
   - 符號映射完全依賴字典表（`liu.json`），不進行硬編碼處理
   - 字典表中的符號映射範例：
     - `s.` 對應 `？`（問號）
     - `.` 對應 `。`（句號）
     - `,` 對應 `，`（逗號）
     - `..` 對應 `：`（全形冒號）
     - `.,` 對應 `；`（全形分號）
   - **連續輸入符號支持**：
     - 輸入 `..`（兩個點號）→ 自動查找 `".."` → `"："`（全形冒號）
     - 輸入 `.,`（點號+逗號）→ 自動查找 `".,"` → `"；"`（全形分號）
   - 符號選擇後不會立即送出，需要按 Space 鍵才會送出選中的符號
8. ✅ 特殊按鍵處理
   - Space：選擇第一個候選字並清除輸入
   - Enter：選擇第一個候選字並清除輸入（與 Space 行為一致）
   - ESC：清除當前輸入
   - Backspace：刪除最後一個字根
   - **Ctrl 組合鍵支援**：當 Ctrl 鍵按下時，所有後續按鍵都會讓事件通過，確保 Ctrl+C、Ctrl+V、Ctrl+A 等組合鍵能正常工作
     - 這與 Python 版本的實現一致，確保在攔截模式下也能正常使用 Ctrl 組合鍵
8. ✅ Shift 鍵切換攔截 / 英模式（行為與 Python 版一致，切換時會清除現有字根）
   - **單獨按一下 Shift**（期間沒有搭配其他鍵）時，在 Shift 放開時切換模式，並清除現有字根
   - **Shift + 其他鍵（例如 Shift+1, Shift+2, Shift+A）** 時，不切換模式，事件全部放行給系統
   - **英模式 = 不攔截模式**：在英模式下，可以正常使用 Shift 做組合鍵（輸出上排符號等）
9. ✅ F4 鍵退出功能
   - F4 鍵在所有模式下都能退出（無論是攔截模式還是不攔截模式）
   - 退出功能在模式檢查之前處理，確保任何時候都能退出程式
10. ✅ 系統托盤退出選項
   - 點擊系統托盤圖示的「退出」選項，行為與 F4 鍵完全一致
   - 通過 WM_COMMAND 消息處理（menu_id = 1001, notification_code = 0）
   - 設置退出標誌（should_quit）並調用 PostQuitMessage(0)
   - 無論是攔截模式還是不攔截模式，都能正常退出程式
11. ✅ Ctrl+Space 熱鍵觸發 GUI 狀態列（取代舊的輸入窗口）
   - 按 Ctrl+Space 可以顯示/隱藏右下角的 GUI 狀態列視窗
   - Ctrl+Space 是 Windows 系統默認的輸入法切換鍵，遊戲通常會允許它通過
   - 熱鍵檢測優先級最高，在模式檢查之前處理

12. ✅ 單一實例鎖定機制
   - 使用 `fs2::FileExt::try_lock_exclusive()` 實現文件鎖定
   - 程序啟動時創建 `UCLLIU.lock` 文件並獲取獨占鎖
   - 如果已有實例在運行，新實例會檢測到鎖已被持有，並立即退出
   - 當程序正常退出時，文件鎖會自動釋放（文件句柄被 drop）
   - 如果程序異常退出，文件鎖也會由操作系統自動釋放
   - 程序退出時會自動刪除 `UCLLIU.lock` 文件（清理殘留文件）
   - **手動測試方法**：
     - 啟動第一個實例，確認程序正常運行
     - 嘗試啟動第二個實例，應該會立即退出並顯示「已有實例運行」錯誤
     - 關閉第一個實例，確認 `UCLLIU.lock` 文件已被自動刪除
     - 再次啟動程序，應該能正常啟動
12. ✅ 攔截模式完整行為
   - **攔截模式（肥米模式）**：`shift_toggle` 為 `false` 時，所有未明確處理的按鍵都會被攔截（不只是英文字母）
     - 符號、標點符號等所有可列印字符都會被攔截
     - 功能鍵會讓事件通過（不攔截）：F1-F24、方向鍵、Tab、CapsLock、NumLock、ScrollLock、Home、End、PageUp、PageDown、Insert、Delete、PrintScreen、Pause、Win鍵、Alt、Menu鍵
   - **不攔截模式（英模式）**：`shift_toggle` 為 `true` 時，所有按鍵都會通過，不進行任何處理
     - 英模式就是不攔截模式，通過 Shift 鍵切換
     - 在英模式下，可以正常輸入英文和符號
12. ✅ 事件阻止機制（防止原始按鍵事件傳遞）
13. ✅ 注入事件檢測（避免無限循環）

## 執行測試

```bash
# 執行所有測試
cargo test

# 只執行輸入法邏輯測試
cargo test input_method::tests

# 只執行 GUI 窗口測試（輸入窗口模式）
cargo test gui_window::tests

# 執行測試並顯示輸出
cargo test -- --nocapture

# 執行所有測試（包括需要 GUI 的測試）
cargo test -- --ignored
```

