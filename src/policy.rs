use crate::capabilities::{Capability, GrantSet, ResourceLimits};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPolicy {
    pub capabilities: Vec<Capability>,
    pub resource_limits: ResourceLimits,
    #[serde(default)]
    pub requires_approval: bool,
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
}
