pub mod benchmark;
pub mod capabilities;
pub mod execution_state;
pub mod mcp;
pub mod policy;
pub mod sandbox;

pub use capabilities::{Capability, GrantSet, ResourceLimits};
pub use execution_state::{ExecutionRecord, ExecutionState, ExecutionStateManager};
pub use policy::Policy;
pub use sandbox::Aegis;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AegisError {
    #[error("Script execution error: {0}")]
    Execution(String),
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),
    #[error("Policy error: {0}")]
    Policy(String),
    #[error("Tool error: {0}")]
    Tool(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
