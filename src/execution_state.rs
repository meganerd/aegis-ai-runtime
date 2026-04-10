use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: String,
    pub code: String,
    pub tool: String,
    pub state: ExecutionState,
    pub result: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Default)]
pub struct ExecutionStateManager {
    executions: Arc<RwLock<HashMap<String, ExecutionRecord>>>,
}

impl ExecutionStateManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&self, code: &str, tool: &str) -> String {
        let id = format!(
            "exec_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let record = ExecutionRecord {
            id: id.clone(),
            code: code.to_string(),
            tool: tool.to_string(),
            state: ExecutionState::Pending,
            result: None,
            created_at: now,
            updated_at: now,
        };

        self.executions.write().unwrap().insert(id.clone(), record);
        id
    }

    pub fn start(&self, id: &str) -> Result<(), String> {
        let mut executions = self.executions.write().unwrap();
        if let Some(exec) = executions.get_mut(id) {
            exec.state = ExecutionState::Running;
            exec.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(())
        } else {
            Err(format!("Execution not found: {}", id))
        }
    }

    pub fn pause(&self, id: &str) -> Result<(), String> {
        let mut executions = self.executions.write().unwrap();
        if let Some(exec) = executions.get_mut(id) {
            if exec.state == ExecutionState::Running {
                exec.state = ExecutionState::Paused;
                exec.updated_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                Ok(())
            } else {
                Err(format!(
                    "Can only pause Running executions, current: {:?}",
                    exec.state
                ))
            }
        } else {
            Err(format!("Execution not found: {}", id))
        }
    }

    pub fn resume(&self, id: &str) -> Result<(), String> {
        let mut executions = self.executions.write().unwrap();
        if let Some(exec) = executions.get_mut(id) {
            if exec.state == ExecutionState::Paused {
                exec.state = ExecutionState::Running;
                exec.updated_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                Ok(())
            } else {
                Err(format!(
                    "Can only resume Paused executions, current: {:?}",
                    exec.state
                ))
            }
        } else {
            Err(format!("Execution not found: {}", id))
        }
    }

    pub fn complete(&self, id: &str, result: &str) -> Result<(), String> {
        let mut executions = self.executions.write().unwrap();
        if let Some(exec) = executions.get_mut(id) {
            exec.state = ExecutionState::Completed;
            exec.result = Some(result.to_string());
            exec.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(())
        } else {
            Err(format!("Execution not found: {}", id))
        }
    }

    pub fn fail(&self, id: &str, error: &str) -> Result<(), String> {
        let mut executions = self.executions.write().unwrap();
        if let Some(exec) = executions.get_mut(id) {
            exec.state = ExecutionState::Failed(error.to_string());
            exec.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(())
        } else {
            Err(format!("Execution not found: {}", id))
        }
    }

    pub fn get(&self, id: &str) -> Option<ExecutionRecord> {
        self.executions.read().unwrap().get(id).cloned()
    }

    pub fn list(&self) -> Vec<ExecutionRecord> {
        self.executions.read().unwrap().values().cloned().collect()
    }
}
