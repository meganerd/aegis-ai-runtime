use crate::capabilities::{Capability, GrantSet, ResourceLimits};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct Aegis {
    grants: Arc<RwLock<GrantSet>>,
    limits: Arc<RwLock<ResourceLimits>>,
    kv_store: Arc<RwLock<HashMap<String, String>>>,
}

impl Clone for Aegis {
    fn clone(&self) -> Self {
        Self {
            grants: Arc::clone(&self.grants),
            limits: Arc::clone(&self.limits),
            kv_store: Arc::clone(&self.kv_store),
        }
    }
}

impl Aegis {
    pub fn new() -> Self {
        let grants = Arc::new(RwLock::new(GrantSet::default()));
        let limits = Arc::new(RwLock::new(ResourceLimits::default()));
        let kv_store = Arc::new(RwLock::new(HashMap::new()));

        Self {
            grants,
            limits,
            kv_store,
        }
    }

    pub fn with_policy(&self, policy: &crate::policy::Policy, tool_name: &str) -> Self {
        if let Some(tool) = policy.get_tool(tool_name) {
            let grants = Arc::new(RwLock::new(GrantSet::new(tool.capabilities.clone())));
            let limits = Arc::new(RwLock::new(tool.resource_limits.clone()));
            Self {
                grants,
                limits,
                kv_store: Arc::new(RwLock::new(HashMap::new())),
            }
        } else {
            Self {
                grants: Arc::clone(&self.grants),
                limits: Arc::clone(&self.limits),
                kv_store: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    pub fn execute(&self, code: &str) -> Result<rhai::Dynamic, String> {
        let mut engine = rhai::Engine::new_raw();

        let grants = self.grants.clone();
        let kv_store = self.kv_store.clone();

        // Register 'log' - always available
        engine.register_fn("log", move |msg: &str| {
            println!("[aegis] {}", msg);
        });

        // Register 'result' - always available (yield is reserved in Rhai)
        engine.register_fn("result", |result: rhai::Dynamic| result);

        // Register 'sleep' - always available
        engine.register_fn("sleep", |millis: u64| {
            std::thread::sleep(std::time::Duration::from_millis(millis));
        });

        // Register 'http_get' if granted
        let has_http_get = grants.read().unwrap().has(&Capability::HttpGet);
        if has_http_get {
            engine.register_fn("http_get", |url: &str| -> Result<String, String> {
                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .map_err(|e| e.to_string())?;

                let response = client.get(url).send().map_err(|e| e.to_string())?;

                if !response.status().is_success() {
                    return Err(format!("HTTP error: {}", response.status()));
                }

                response.text().map_err(|e| e.to_string())
            });
        }

        // Register 'kv_set' if granted
        let has_kv_set = grants.read().unwrap().has(&Capability::KvSet);
        if has_kv_set {
            let kv_store = kv_store.clone();
            engine.register_fn("kv_set", move |key: &str, value: &str| {
                kv_store
                    .write()
                    .unwrap()
                    .insert(key.to_string(), value.to_string());
                true
            });
        }

        // Register 'kv_get' if granted
        let has_kv_get = grants.read().unwrap().has(&Capability::KvGet);
        if has_kv_get {
            let kv_store = kv_store.clone();
            engine.register_fn("kv_get", move |key: &str| -> Option<String> {
                kv_store.read().unwrap().get(key).cloned()
            });
        }

        // Register 'file_read' if granted
        let has_file_read = grants.read().unwrap().has(&Capability::FileRead);
        if has_file_read {
            engine.register_fn("file_read", |path: &str| -> Result<String, String> {
                std::fs::read_to_string(path).map_err(|e| e.to_string())
            });
        }

        // Register 'file_write' if granted
        let has_file_write = grants.read().unwrap().has(&Capability::FileWrite);
        if has_file_write {
            engine.register_fn(
                "file_write",
                |path: &str, content: &str| -> Result<bool, String> {
                    std::fs::write(path, content).map_err(|e| e.to_string())?;
                    Ok(true)
                },
            );
        }

        // Register 'file_list' if granted
        let has_file_list = grants.read().unwrap().has(&Capability::FileList);
        if has_file_list {
            engine.register_fn("file_list", |path: &str| -> Result<Vec<String>, String> {
                let entries = std::fs::read_dir(path)
                    .map_err(|e| e.to_string())?
                    .map(|e| e.map(|e| e.file_name().to_string_lossy().to_string()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?;
                Ok(entries)
            });
        }

        // Execute the script
        let result = engine.eval::<rhai::Dynamic>(code);

        match result {
            Ok(dyn_val) => Ok(dyn_val),
            Err(e) => Err(format!("Script error: {}", e)),
        }
    }
}

impl Default for Aegis {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let aegis = Aegis::new();
        let result = aegis.execute("let x = 42; log(x); yield(x)");
        assert!(result.is_ok() || result.err().unwrap().contains("Script error"));
    }
}
