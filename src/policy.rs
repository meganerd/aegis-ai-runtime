use crate::capabilities::{Capability, ResourceLimits};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolPolicy {
    pub capabilities: Vec<Capability>,
    pub resource_limits: ResourceLimits,
    #[serde(default)]
    pub requires_approval: bool,
    #[serde(default)]
    pub allowed_paths: Vec<String>,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default)]
    pub allowed_key_prefixes: Vec<String>,
    #[serde(default)]
    pub allowed_env_vars: Vec<String>,
}

impl ToolPolicy {
    pub fn allows_path(&self, path: &str) -> bool {
        if self.allowed_paths.is_empty() {
            return true;
        }
        self.allowed_paths
            .iter()
            .any(|p| path.starts_with(p) || path.contains(p))
    }

    pub fn allows_domain(&self, url: &str) -> bool {
        if self.allowed_domains.is_empty() {
            return true;
        }
        self.allowed_domains.iter().any(|d| url.contains(d))
    }

    pub fn allows_key(&self, key: &str) -> bool {
        if self.allowed_key_prefixes.is_empty() {
            return true;
        }
        self.allowed_key_prefixes.iter().any(|p| key.starts_with(p))
    }

    pub fn allows_env(&self, name: &str) -> bool {
        if self.allowed_env_vars.is_empty() {
            return true;
        }
        self.allowed_env_vars.iter().any(|n| n == name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    #[serde(default)]
    pub default_capabilities: Vec<String>,
    pub tools: HashMap<String, ToolPolicy>,
}

impl Policy {
    pub fn from_yaml(content: &str) -> Result<Self, crate::AegisError> {
        serde_yaml::from_str(content).map_err(|e| crate::AegisError::Policy(e.to_string()))
    }

    pub fn get_tool(&self, name: &str) -> Option<&ToolPolicy> {
        self.tools.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_POLICY_YAML: &str = r#"
default_capabilities:
  - log
  - yield

tools:
  weather_fetch:
    capabilities:
      - http_get
      - kv_set
    resource_limits:
      memory_mb: 32
      max_operations: 1000000
      timeout_seconds: 30
    requires_approval: false
"#;

    #[test]
    fn test_parse_policy() {
        let policy = Policy::from_yaml(TEST_POLICY_YAML).unwrap();
        assert_eq!(policy.default_capabilities, vec!["log", "yield"]);

        // The YAML doesn't have all fields so they'll get defaults
        // Just check tool exists
        let tool = policy.get_tool("weather_fetch");
        assert!(tool.is_some());
    }

    #[test]
    fn test_env_allowlist() {
        let tool = ToolPolicy {
            allowed_env_vars: vec!["API_TOKEN".to_string()],
            ..Default::default()
        };

        assert!(tool.allows_env("API_TOKEN"));
        assert!(!tool.allows_env("SSH_PRIVATE_KEY"));
    }
}
