use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    #[serde(rename = "http_get")]
    HttpGet,
    #[serde(rename = "http_post")]
    HttpPost,
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "file_read")]
    FileRead,
    #[serde(rename = "file_write")]
    FileWrite,
    #[serde(rename = "file_list")]
    FileList,
    #[serde(rename = "kv_get")]
    KvGet,
    #[serde(rename = "kv_set")]
    KvSet,
    #[serde(rename = "exec")]
    Exec,
    #[serde(rename = "env")]
    Env,
}

impl Capability {
    pub fn name(&self) -> &'static str {
        match self {
            Capability::HttpGet => "http_get",
            Capability::HttpPost => "http_post",
            Capability::Http => "http",
            Capability::FileRead => "file_read",
            Capability::FileWrite => "file_write",
            Capability::FileList => "file_list",
            Capability::KvGet => "kv_get",
            Capability::KvSet => "kv_set",
            Capability::Exec => "exec",
            Capability::Env => "env",
        }
    }

    pub fn matches(&self, other: &Capability) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrantSet {
    caps: Vec<Capability>,
}

impl GrantSet {
    pub fn new(caps: Vec<Capability>) -> Self {
        Self { caps }
    }

    pub fn has(&self, cap: &Capability) -> bool {
        self.caps.iter().any(|c| c.matches(cap))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    #[serde(default = "default_max_memory")]
    pub max_memory_mb: usize,
    #[serde(default = "default_max_operations")]
    pub max_operations: u64,
    #[serde(default = "default_max_call_depth")]
    pub max_call_depth: u32,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

fn default_max_memory() -> usize {
    64
}
fn default_max_operations() -> u64 {
    1_000_000
}
fn default_max_call_depth() -> u32 {
    64
}
fn default_timeout() -> u64 {
    30
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 64,
            max_operations: 1_000_000,
            max_call_depth: 64,
            timeout_seconds: 30,
        }
    }
}
