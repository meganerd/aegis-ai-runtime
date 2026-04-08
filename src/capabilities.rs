use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    HttpGet,
    HttpPost,
    Http { method: String },
    FileRead { path_prefix: Vec<String> },
    FileWrite { path_prefix: Vec<String> },
    FileList { path_prefix: Vec<String> },
    KvGet,
    KvSet { key_prefix: Vec<String> },
    Exec { allowed_commands: Vec<String> },
    Env { allowed_vars: Vec<String> },
}

impl Capability {
    pub fn name(&self) -> &'static str {
        match self {
            Capability::HttpGet => "http_get",
            Capability::HttpPost => "http_post",
            Capability::Http { .. } => "http",
            Capability::FileRead { .. } => "file_read",
            Capability::FileWrite { .. } => "file_write",
            Capability::FileList { .. } => "file_list",
            Capability::KvGet => "kv_get",
            Capability::KvSet { .. } => "kv_set",
            Capability::Exec { .. } => "exec",
            Capability::Env { .. } => "env",
        }
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
        self.caps.iter().any(|c| match (c, cap) {
            (Capability::HttpGet, Capability::HttpGet) => true,
            (Capability::HttpPost, Capability::HttpPost) => true,
            (Capability::Http { .. }, Capability::Http { .. }) => true,
            (
                Capability::FileRead { path_prefix: p1 },
                Capability::FileRead { path_prefix: p2 },
            ) => p2.iter().any(|p| p1.iter().any(|pf| p.starts_with(pf))),
            (
                Capability::FileWrite { path_prefix: p1 },
                Capability::FileWrite { path_prefix: p2 },
            ) => p2.iter().any(|p| p1.iter().any(|pf| p.starts_with(pf))),
            (
                Capability::FileList { path_prefix: p1 },
                Capability::FileList { path_prefix: p2 },
            ) => p2.iter().any(|p| p1.iter().any(|pf| p.starts_with(pf))),
            (Capability::KvGet, Capability::KvGet) => true,
            (Capability::KvSet { key_prefix: p1 }, Capability::KvSet { key_prefix: p2 }) => {
                p2.iter().any(|p| p1.iter().any(|pf| p.starts_with(pf)))
            }
            (
                Capability::Exec {
                    allowed_commands: a1,
                },
                Capability::Exec {
                    allowed_commands: a2,
                },
            ) => a2.iter().any(|c| a1.contains(c)),
            (Capability::Env { allowed_vars: a1 }, Capability::Env { allowed_vars: a2 }) => {
                a2.iter().any(|v| a1.contains(v))
            }
            _ => false,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: usize,
    pub max_operations: u64,
    pub max_call_depth: u32,
    pub timeout_seconds: u64,
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
