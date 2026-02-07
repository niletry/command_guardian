use crate::models::{TaskConfig, TaskStatus};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct RunningProcess {
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    pub child: Box<dyn portable_pty::Child + Send>,
    pub writer: Box<dyn Write + Send>,
    pub kill_tx: std::sync::mpsc::Sender<()>,
}

pub struct AppState {
    pub tasks: Arc<Mutex<HashMap<String, TaskConfig>>>,
    pub processes: Arc<Mutex<HashMap<String, RunningProcess>>>,
    pub statuses: Arc<Mutex<HashMap<String, TaskStatus>>>,
    pub log_dir: PathBuf,
    pub config_path: PathBuf,
}

impl AppState {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let data_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("data"));

        let _ = std::fs::create_dir_all(&data_dir);
        let log_dir = data_dir.join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        let config_path = data_dir.join("config.json");

        let tasks = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            serde_json::from_str::<Vec<TaskConfig>>(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        let mut tasks_map = HashMap::new();
        let mut statuses_map = HashMap::new();

        for config in tasks {
            let id = config.id.clone();
            statuses_map.insert(
                id.clone(),
                TaskStatus {
                    id: id.clone(),
                    status: "stopped".to_string(),
                    pid: None,
                    start_time: None,
                },
            );
            tasks_map.insert(id, config);
        }

        Self {
            tasks: Arc::new(Mutex::new(tasks_map)),
            processes: Arc::new(Mutex::new(HashMap::new())),
            statuses: Arc::new(Mutex::new(statuses_map)),
            log_dir,
            config_path,
        }
    }

    pub fn save_config(&self) {
        let tasks_map = self.tasks.lock().unwrap();
        let tasks_vec: Vec<TaskConfig> = tasks_map.values().cloned().collect();
        if let Ok(content) = serde_json::to_string_pretty(&tasks_vec) {
            let _ = std::fs::write(&self.config_path, content);
        }
    }
}

pub fn stop_task_internal(state: &AppState, id: &str) -> bool {
    println!(">>> BACKEND: stop_task_internal id={}", id);
    let mut processes = state.processes.lock().unwrap();
    let was_running = if let Some(mut proc) = processes.remove(id) {
        let _ = proc.child.kill();
        let _ = proc.kill_tx.send(());
        true
    } else {
        false
    };

    let mut statuses = state.statuses.lock().unwrap();
    if let Some(s) = statuses.get_mut(id) {
        s.status = "stopped".to_string();
        s.pid = None;
        s.start_time = None;
    }

    was_running
}
