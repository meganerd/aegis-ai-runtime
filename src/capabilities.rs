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

    pub fn is_http(&self) -> bool {
        matches!(self, Capability::HttpGet | Capability::HttpPost)
    }

    pub fn is_file_read(&self) -> bool {
        matches!(self, Capability::FileRead)
    }

    pub fn is_file_write(&self) -> bool {
        matches!(self, Capability::FileWrite)
    }

    pub fn is_file_list(&self) -> bool {
        matches!(self, Capability::FileList)
    }

    pub fn is_kv_set(&self) -> bool {
        matches!(self, Capability::KvSet)
    }

    pub fn is_kv_get(&self) -> bool {
        matches!(self, Capability::KvGet)
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

    pub fn has_http(&self) -> bool {
        self.caps.iter().any(|c| c.is_http())
    }

    pub fn has_file_read(&self) -> bool {
        self.caps.iter().any(|c| c.is_file_read())
    }

    pub fn has_file_write(&self) -> bool {
        self.caps.iter().any(|c| c.is_file_write())
    }

    pub fn has_file_list(&self) -> bool {
        self.caps.iter().any(|c| c.is_file_list())
    }

    pub fn has_kv_set(&self) -> bool {
        self.caps.iter().any(|c| c.is_kv_set())
    }

    pub fn has_kv_get(&self) -> bool {
        self.caps.iter().any(|c| c.is_kv_get())
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
            max_memory_mb: default_max_memory(),
            max_operations: default_max_operations(),
            max_call_depth: default_max_call_depth(),
            timeout_seconds: default_timeout(),
        }
    }
}
