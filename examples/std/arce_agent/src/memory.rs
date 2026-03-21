// Persistent memory backed by the filesystem.
//
// Stores key-value pairs as JSON. On ArceOS this uses the FAT filesystem
// mounted from the virtio-blk device. On Linux it uses a local file.

use std::collections::BTreeMap;

use log::{error, info};

const MEMORY_PATH: &str = "/memory.json";

pub struct Memory {
    data: BTreeMap<String, String>,
}

impl Memory {
    /// Load memory from the filesystem. Returns empty memory if file doesn't exist.
    pub fn load() -> Self {
        let data = match std::fs::read_to_string(MEMORY_PATH) {
            Ok(contents) => match serde_json::from_str::<BTreeMap<String, String>>(&contents) {
                Ok(map) => {
                    info!("[memory] Loaded {} entries from {}", map.len(), MEMORY_PATH);
                    map
                }
                Err(e) => {
                    error!(
                        "[memory] Parse error in {}: {}, starting fresh",
                        MEMORY_PATH, e
                    );
                    BTreeMap::new()
                }
            },
            Err(_) => {
                error!(
                    "[memory] No memory file found at {}, starting fresh",
                    MEMORY_PATH
                );
                BTreeMap::new()
            }
        };
        Self { data }
    }

    /// Save memory to the filesystem.
    fn save(&self) {
        match serde_json::to_string_pretty(&self.data) {
            Ok(json) => {
                if let Err(e) = std::fs::write(MEMORY_PATH, &json) {
                    error!("[memory] Failed to write {}: {}", MEMORY_PATH, e);
                } else {
                    info!(
                        "[memory] Saved {} entries to {}",
                        self.data.len(),
                        MEMORY_PATH
                    );
                }
            }
            Err(e) => error!("[memory] Serialize error: {}", e),
        }
    }

    /// Read all memory entries as a formatted string.
    pub fn read_all(&self) -> String {
        if self.data.is_empty() {
            return "记忆为空，尚未保存任何内容。".to_string();
        }
        let mut out = String::from("当前记忆内容:\n");
        for (k, v) in &self.data {
            out.push_str(&format!("  - {}: {}\n", k, v));
        }
        out
    }

    /// Write a key-value pair and persist.
    pub fn write(&mut self, key: &str, value: &str) -> String {
        self.data.insert(key.to_string(), value.to_string());
        self.save();
        format!("已保存记忆: {} = {}", key, value)
    }

    /// Delete a key and persist.
    pub fn delete(&mut self, key: &str) -> String {
        if self.data.remove(key).is_some() {
            self.save();
            format!("已删除记忆: {}", key)
        } else {
            format!("记忆中没有找到键: {}", key)
        }
    }

    /// Get formatted memory summary for system prompt injection.
    pub fn summary(&self) -> String {
        if self.data.is_empty() {
            return String::new();
        }
        let mut out = String::from("\n\n[已加载的持久记忆]\n");
        for (k, v) in &self.data {
            out.push_str(&format!("- {}: {}\n", k, v));
        }
        out
    }
}
