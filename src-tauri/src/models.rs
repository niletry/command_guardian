use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TaskConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    pub tag: String,
    pub auto_retry: bool,
    pub env_vars: Option<HashMap<String, String>>,
}

#[derive(Clone, Serialize, Debug)]
pub struct TaskStatus {
    pub id: String,
    pub status: String,
    pub pid: Option<u32>,
    pub start_time: Option<u64>,
}

#[derive(Serialize)]
pub struct TaskView {
    pub config: TaskConfig,
    pub status: TaskStatus,
}
