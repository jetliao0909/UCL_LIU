//! 字碼表字典模組

use anyhow::{Context, Result};
use log::{info, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

/// 字碼表字典
#[derive(Clone)]
pub struct Dictionary {
    /// 字根 -> 候選字列表的映射
    pub code_to_chars: HashMap<String, Vec<String>>,
    /// 同音字表（可選）
    pub pinyi_data: Option<Vec<String>>,
}

impl Dictionary {
    /// 載入字碼表
    /// 字典檔必須與執行檔放在同一目錄
    pub fn load() -> Result<Self> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "無法取得執行檔目錄"
            ))?;
        
        // 字典檔必須與執行檔放在同一目錄
        let json_path = exe_dir.join("liu.json");
        
        if !json_path.exists() {
            return Err(anyhow::anyhow!(
                "找不到字碼表檔案 liu.json\n請確保 liu.json 與執行檔放在同一目錄\n執行檔目錄: {:?}",
                exe_dir
            ));
        }
        
        info!("載入字碼表: {:?}", json_path);
        
        let content = fs::read_to_string(&json_path)
            .with_context(|| format!("無法讀取字碼表: {:?}", json_path))?;
        
        // JSON 檔案格式：{ "chardefs": { "字根": ["候選字1", "候選字2", ...], ... } }
        #[derive(Deserialize)]
        struct LiuJsonFile {
            chardefs: HashMap<String, Vec<String>>,
        }
        
        let json_file: LiuJsonFile = serde_json::from_str(&content)
            .with_context(|| "無法解析 JSON 格式")?;
        
        // 提取 chardefs 並將所有鍵轉為小寫（根據 Python 版本的處理邏輯）
        // 參考：uclliu.pyw 第 1180-1189 行
        let mut code_map: HashMap<String, Vec<String>> = HashMap::new();
        for (key, value) in json_file.chardefs {
            let lower_key = key.to_lowercase();
            // 如果已經存在小寫鍵，合併候選字列表
            code_map.entry(lower_key)
                .and_modify(|v| {
                    // 合併候選字，避免重複
                    for char in &value {
                        if !v.contains(char) {
                            v.push(char.clone());
                        }
                    }
                })
                .or_insert_with(|| value);
        }
        
        info!("已載入 {} 個字根", code_map.len());
        
        // 載入同音字表（可選）
        // 同音字表必須與執行檔放在同一目錄
        let pinyi_path = exe_dir.join("pinyi.txt");
        
        let pinyi_data = if pinyi_path.exists() {
            info!("載入同音字表: {:?}", pinyi_path);
            Some(
                fs::read_to_string(&pinyi_path)
                    .ok()
                    .map(|s| s.lines().map(|l| l.to_string()).collect())
                    .unwrap_or_default()
            )
        } else {
            None
        };
        
        Ok(Self {
            code_to_chars: code_map,
            pinyi_data,
        })
    }
    
    /// 根據字根查詢候選字
    pub fn lookup(&self, code: &str) -> Option<&Vec<String>> {
        self.code_to_chars.get(code)
    }
    
    /// 取得候選字數量
    pub fn get_candidate_count(&self, code: &str) -> usize {
        self.lookup(code).map(|v| v.len()).unwrap_or(0)
    }
    
    /// 檢查是否存在以指定字根開頭的字根（用於補碼機制判斷）
    /// 例如：檢查是否存在以 "sis" 開頭的字根（如 "sisp"）
    pub fn has_prefix(&self, prefix: &str) -> bool {
        self.code_to_chars.keys().any(|key| key.starts_with(prefix) && key != prefix)
    }
}

