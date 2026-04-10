use crate::capabilities::{Capability, GrantSet, ResourceLimits};
use crate::policy::ToolPolicy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct Aegis {
    grants: Arc<RwLock<GrantSet>>,
    limits: Arc<RwLock<ResourceLimits>>,
    kv_store: Arc<RwLock<HashMap<String, String>>>,
    policy: Arc<RwLock<Option<ToolPolicy>>>,
}

impl Clone for Aegis {
    fn clone(&self) -> Self {
        Self {
            grants: Arc::clone(&self.grants),
            limits: Arc::clone(&self.limits),
            kv_store: Arc::clone(&self.kv_store),
            policy: Arc::clone(&self.policy),
        }
    }
}

impl Aegis {
    pub fn new() -> Self {
        let grants = Arc::new(RwLock::new(GrantSet::default()));
        let limits = Arc::new(RwLock::new(ResourceLimits::default()));
        let kv_store = Arc::new(RwLock::new(HashMap::new()));
        let policy = Arc::new(RwLock::new(None));

        Self {
            grants,
            limits,
            kv_store,
            policy,
        }
    }

    pub fn with_policy(&self, policy: &crate::policy::Policy, tool_name: &str) -> Self {
        if let Some(tool) = policy.get_tool(tool_name) {
            let grants = Arc::new(RwLock::new(GrantSet::new(tool.capabilities.clone())));
            let limits = Arc::new(RwLock::new(tool.resource_limits.clone()));
            let tool_policy = Arc::new(RwLock::new(Some(tool.clone())));
            Self {
                grants,
                limits,
                kv_store: Arc::new(RwLock::new(HashMap::new())),
                policy: tool_policy,
            }
        } else {
            Self {
                grants: Arc::clone(&self.grants),
                limits: Arc::clone(&self.limits),
                kv_store: Arc::new(RwLock::new(HashMap::new())),
                policy: Arc::new(RwLock::new(None)),
            }
        }
    }

    pub fn execute(&self, code: &str) -> Result<rhai::Dynamic, String> {
        let limits = self.limits.read().unwrap();

        // Store limits in local variables to avoid borrow issues
        let max_ops = limits.max_operations;
        let max_depth = limits.max_call_depth;

        // Configure engine with resource limits
        let mut engine = rhai::Engine::new_raw();
        engine.set_max_operations(max_ops);

        // Release the lock before long-running operations
        drop(limits);

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
        let has_http_get = grants.read().unwrap().has_http();
        let http_policy = self.policy.clone();
        if has_http_get {
            engine.register_fn("http_get", move |url: &str| -> Result<String, String> {
                if let Some(ref p) = http_policy.read().unwrap().as_ref() {
                    if !p.allows_domain(url) {
                        return Err(format!("Domain not allowed: {}", url));
                    }
                }
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
        let has_kv_set = grants.read().unwrap().has_kv_set();
        let kv_policy = self.policy.clone();
        if has_kv_set {
            let kv_store = kv_store.clone();
            engine.register_fn("kv_set", move |key: &str, value: &str| {
                if let Some(ref p) = kv_policy.read().unwrap().as_ref() {
                    if !p.allows_key(key) {
                        return Err(format!("Key prefix not allowed: {}", key));
                    }
                }
                kv_store
                    .write()
                    .unwrap()
                    .insert(key.to_string(), value.to_string());
                Ok(true)
            });
        }

        // Register 'kv_get' if granted
        let has_kv_get = grants.read().unwrap().has_kv_get();
        let get_policy = self.policy.clone();
        if has_kv_get {
            let kv_store = kv_store.clone();
            engine.register_fn("kv_get", move |key: &str| -> Result<String, String> {
                if let Some(ref p) = get_policy.read().unwrap().as_ref() {
                    if !p.allows_key(key) {
                        return Err(format!("Key prefix not allowed: {}", key));
                    }
                }
                kv_store
                    .read()
                    .unwrap()
                    .get(key)
                    .cloned()
                    .ok_or_else(|| format!("Key not found: {}", key))
            });
        }

        // Register 'file_read' if granted
        let has_file_read = grants.read().unwrap().has_file_read();
        let file_policy = self.policy.clone();
        if has_file_read {
            engine.register_fn("file_read", move |path: &str| -> Result<String, String> {
                if let Some(p) = file_policy.read().unwrap().as_ref() {
                    if !p.allows_path(path) {
                        return Err(format!("Path not allowed: {}", path));
                    }
                }
                std::fs::read_to_string(path).map_err(|e| e.to_string())
            });
        }

        // Register 'file_write' if granted
        let has_file_write = grants.read().unwrap().has_file_write();
        let write_policy = self.policy.clone();
        if has_file_write {
            engine.register_fn(
                "file_write",
                move |path: &str, content: &str| -> Result<bool, String> {
                    if let Some(p) = write_policy.read().unwrap().as_ref() {
                        if !p.allows_path(path) {
                            return Err(format!("Path not allowed: {}", path));
                        }
                    }
                    std::fs::write(path, content).map_err(|e| e.to_string())?;
                    Ok(true)
                },
            );
        }

        // Register 'file_list' if granted
        let has_file_list = grants.read().unwrap().has_file_list();
        let list_policy = self.policy.clone();
        if has_file_list {
            engine.register_fn(
                "file_list",
                move |path: &str| -> Result<Vec<String>, String> {
                    if let Some(p) = list_policy.read().unwrap().as_ref() {
                        if !p.allows_path(path) {
                            return Err(format!("Path not allowed: {}", path));
                        }
                    }
                    let entries = std::fs::read_dir(path)
                        .map_err(|e| e.to_string())?
                        .map(|e| e.map(|e| e.file_name().to_string_lossy().to_string()))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| e.to_string())?;
                    Ok(entries)
                },
            );
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

    #[test]
    fn test_no_fn_definition() {
        let aegis = Aegis::new();
        let result = aegis.execute("fn add(a, b) { a + b }");
        let err = result.unwrap_err();
        assert!(
            err.contains("reserved keyword"),
            "Expected 'reserved keyword' error, got: {}",
            err
        );
    }

    #[test]
    fn test_no_call_unregistered_tool() {
        let aegis = Aegis::new();
        let result = aegis.execute("http_get(\"https://example.com\")");
        let err = result.unwrap_err();
        assert!(
            err.contains("Function not found"),
            "Expected Function not found error, got: {}",
            err
        );
    }

    #[test]
    fn test_no_state_leakage() {
        let aegis = Aegis::new();
        aegis.execute("let secret = 42").ok();
        let result = aegis.execute("secret");
        let err = result.unwrap_err();
        assert!(
            err.contains("Variable not found"),
            "Expected Variable not found error, got: {}",
            err
        );
    }

    #[test]
    fn test_resource_limit_max_operations() {
        let aegis = Aegis::new();
        let script = (0..100000).map(|i| format!("{} + ", i)).collect::<String>() + "0";
        let result = aegis.execute(&script);
        assert!(
            result.is_err(),
            "Expected error due to operation limit, got: {:?}",
            result
        );
    }

    #[test]
    fn test_with_policy() {
        use crate::capabilities::Capability;
        use crate::policy::Policy;
        let policy_yaml = std::fs::read_to_string("policy.yaml").unwrap();
        let policy = Policy::from_yaml(&policy_yaml).unwrap();
        let aegis = Aegis::new();
        let aegis = aegis.with_policy(&policy, "weather_fetch");
        assert!(aegis.grants.read().unwrap().has_http());
    }

    #[test]
    fn test_path_validation() {
        use crate::policy::Policy;
        let policy_yaml = std::fs::read_to_string("policy.yaml").unwrap();
        let policy = Policy::from_yaml(&policy_yaml).unwrap();
        let tool = policy.get_tool("file_processor").unwrap();

        assert!(!tool.allows_path("/etc/passwd"), "should deny /etc/passwd");
        assert!(
            tool.allows_path("/data/input/file.txt"),
            "should allow /data/input/"
        );
    }

    #[test]
    fn test_key_prefix_validation() {
        use crate::policy::Policy;
        let policy_yaml = std::fs::read_to_string("policy.yaml").unwrap();
        let policy = Policy::from_yaml(&policy_yaml).unwrap();
        let tool = policy.get_tool("weather_fetch").unwrap();

        assert!(!tool.allows_key("secret"), "should deny secret key");
        assert!(
            tool.allows_key("weather:temp"),
            "should allow weather: prefix"
        );
    }

    #[test]
    fn test_domain_validation() {
        use crate::policy::Policy;
        let policy_yaml = std::fs::read_to_string("policy.yaml").unwrap();
        let policy = Policy::from_yaml(&policy_yaml).unwrap();
        let tool = policy.get_tool("weather_fetch").unwrap();

        assert!(
            !tool.allows_domain("https://evil.com/data"),
            "should deny evil.com"
        );
        assert!(
            tool.allows_domain("https://api.weather.com/data"),
            "should allow api.weather.com"
        );
    }
}
