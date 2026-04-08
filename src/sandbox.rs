use crate::capabilities::{Capability, GrantSet, ResourceLimits};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct Aegis {
    grants: Arc<RwLock<GrantSet>>,
    limits: Arc<RwLock<ResourceLimits>>,
    kv_store: Arc<RwLock<HashMap<String, String>>>,
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

    pub fn with_policy(self, policy: &crate::policy::Policy, tool_name: &str) -> Self {
        if let Some(tool) = policy.get_tool(tool_name) {
            *self.grants.write().unwrap() = GrantSet::new(tool.capabilities.clone());
            *self.limits.write().unwrap() = tool.resource_limits.clone();
        }
        self
    }

    pub fn execute(&self, code: &str) -> Result<rhai::Dynamic, String> {
        // Create a fresh engine for each execution
        let mut engine = rhai::Engine::new_raw();

        let grants = self.grants.clone();
        let kv_store = self.kv_store.clone();

        // Register 'log' - always available
        engine.register_fn("log", move |msg: &str| {
            println!("[aegis] {}", msg);
        });

        // Register 'yield' - always available
        engine.register_fn("yield", |result: rhai::Dynamic| result);

        // Register 'sleep' - always available
        engine.register_fn("sleep", |millis: u64| {
            std::thread::sleep(std::time::Duration::from_millis(millis));
        });

        // Register 'http_get' if granted
        if grants.read().unwrap().has(&Capability::HttpGet) {
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
        if grants
            .read()
            .unwrap()
            .has(&Capability::KvSet { key_prefix: vec![] })
        {
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
        if grants.read().unwrap().has(&Capability::KvGet) {
            let kv_store = kv_store.clone();
            engine.register_fn("kv_get", move |key: &str| -> Option<String> {
                kv_store.read().unwrap().get(key).cloned()
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
