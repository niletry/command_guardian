use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

// --- Data Models ---

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

struct RunningProcess {
    master: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send>,
    writer: Box<dyn Write + Send>,
    kill_tx: std::sync::mpsc::Sender<()>,
}

pub struct AppState {
    tasks: Arc<Mutex<HashMap<String, TaskConfig>>>,
    processes: Arc<Mutex<HashMap<String, RunningProcess>>>,
    statuses: Arc<Mutex<HashMap<String, TaskStatus>>>,
    log_dir: std::path::PathBuf,
    config_path: std::path::PathBuf,
}

impl AppState {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let data_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("data"));

        std::fs::create_dir_all(&data_dir).unwrap_or_default();

        let log_dir = data_dir.join("logs");
        std::fs::create_dir_all(&log_dir).unwrap_or_default();

        let config_path = data_dir.join("config.json");

        // Load initial tasks
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

// --- Commands ---

#[tauri::command]
fn create_task(
    state: State<'_, AppState>,
    name: String,
    command: String,
    tag: String,
    auto_retry: bool,
    env_vars: Option<HashMap<String, String>>,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let config = TaskConfig {
        id: id.clone(),
        name,
        command,
        tag,
        auto_retry,
        env_vars,
    };

    let status = TaskStatus {
        id: id.clone(),
        status: "stopped".to_string(),
        pid: None,
        start_time: None,
    };

    state.tasks.lock().unwrap().insert(id.clone(), config);
    state.statuses.lock().unwrap().insert(id.clone(), status);

    state.save_config();

    id
}

#[tauri::command]
fn get_tasks(state: State<'_, AppState>) -> Vec<TaskView> {
    let tasks_map = state.tasks.lock().unwrap();
    let statuses_map = state.statuses.lock().unwrap();

    let mut views = Vec::new();
    for (id, config) in tasks_map.iter() {
        if let Some(status) = statuses_map.get(id) {
            views.push(TaskView {
                config: config.clone(),
                status: status.clone(),
            });
        }
    }
    views
}

#[tauri::command]
fn delete_task(state: State<'_, AppState>, id: String) {
    let _ = stop_task_internal(&state, id.clone());

    state.tasks.lock().unwrap().remove(&id);
    state.statuses.lock().unwrap().remove(&id);

    state.save_config();
}

fn stop_task_internal(state: &AppState, id: String) -> bool {
    let mut processes = state.processes.lock().unwrap();
    if let Some(mut proc) = processes.remove(&id) {
        let _ = proc.child.kill();
        let _ = proc.kill_tx.send(());

        let mut statuses = state.statuses.lock().unwrap();
        if let Some(s) = statuses.get_mut(&id) {
            s.status = "stopped".to_string();
            s.pid = None;
            s.start_time = None;
        }
        true
    } else {
        false
    }
}

#[tauri::command]
fn start_task(state: State<'_, AppState>, app: AppHandle, id: String) -> Result<(), String> {
    let config = {
        let tasks = state.tasks.lock().unwrap();
        tasks.get(&id).cloned().ok_or("Task not found")?
    };

    {
        let processes = state.processes.lock().unwrap();
        if processes.contains_key(&id) {
            return Ok(());
        }
    }

    let pty_system = NativePtySystem::default();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = CommandBuilder::new("cmd");
        c.args(["/C", &config.command]);
        c
    } else {
        let mut c = CommandBuilder::new("sh");
        c.args(["-c", &config.command]);
        c
    };

    // Apply environment variables
    if let Some(env_vars) = config.env_vars {
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
    }

    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    let pid = child.process_id().unwrap_or(0);

    {
        let mut statuses = state.statuses.lock().unwrap();
        if let Some(s) = statuses.get_mut(&id) {
            s.status = "running".to_string();
            s.pid = Some(pid);
            s.start_time = Some(chrono::Utc::now().timestamp() as u64);
        }
    }
    app.emit("task-updated", id.clone()).unwrap();

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    let (tx, rx) = std::sync::mpsc::channel();
    let task_id_clone = id.clone();
    let app_clone = app.clone();
    let log_path = state.log_dir.join(format!("{}.log", id));

    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        let mut log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .ok();

        loop {
            if rx.try_recv().is_ok() {
                break;
            }
            match reader.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let data = String::from_utf8_lossy(&buffer[..n]).to_string();

                    // Write to file
                    if let Some(ref mut f) = log_file {
                        let _ = f.write_all(&buffer[..n]);
                        let _ = f.flush();
                    }

                    let _ = app_clone.emit(
                        "task-output",
                        serde_json::json!({ "id": task_id_clone, "data": data }),
                    );
                }
                Ok(_) => break,
                Err(_) => break,
            }
        }
    });

    let process = RunningProcess {
        master: pair.master,
        child,
        writer,
        kill_tx: tx,
    };

    state.processes.lock().unwrap().insert(id.clone(), process);

    let processes_arc = state.processes.clone();
    let statuses_arc = state.statuses.clone();
    let id_clone = id.clone();
    let app_clone_2 = app.clone();

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(500));
        let is_alive = {
            let mut processes = processes_arc.lock().unwrap();
            if let Some(proc) = processes.get_mut(&id_clone) {
                match proc.child.try_wait() {
                    Ok(Some(_)) => false,
                    Ok(None) => true,
                    Err(_) => false,
                }
            } else {
                false
            }
        };

        if !is_alive {
            let mut processes = processes_arc.lock().unwrap();
            if processes.contains_key(&id_clone) {
                processes.remove(&id_clone);
                {
                    let mut statuses = statuses_arc.lock().unwrap();
                    if let Some(s) = statuses.get_mut(&id_clone) {
                        s.status = "stopped".to_string();
                        s.pid = None;
                        s.start_time = None;
                    }
                }
                let _ = app_clone_2.emit("task-updated", id_clone.clone());
            }
            break;
        }
    });

    Ok(())
}

#[tauri::command]
fn stop_task(state: State<'_, AppState>, id: String) {
    let _ = stop_task_internal(&state, id);
}

#[tauri::command]
fn resize_pty(state: State<'_, AppState>, id: String, rows: u16, cols: u16) {
    let mut processes = state.processes.lock().unwrap();
    if let Some(proc) = processes.get_mut(&id) {
        let _ = proc.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
    }
}

#[tauri::command]
fn get_log_history(state: State<'_, AppState>, id: String) -> String {
    let log_path = state.log_dir.join(format!("{}.log", id));
    if !log_path.exists() {
        return String::new();
    }

    // Read last 10KB or so to avoid loading massive files
    use std::io::Seek;
    let mut file = match std::fs::File::open(&log_path) {
        Ok(f) => f,
        Err(_) => return String::new(),
    };

    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    let read_size = std::cmp::min(size, 50_000); // ~50KB

    if let Err(_) = file.seek(std::io::SeekFrom::End(-(read_size as i64))) {
        let _ = file.seek(std::io::SeekFrom::Start(0));
    }

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap_or_default();
    String::from_utf8_lossy(&buffer).to_string()
}

#[tauri::command]
fn clear_log_history(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let log_path = state.log_dir.join(format!("{}.log", id));
    if log_path.exists() {
        std::fs::remove_file(log_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn write_to_pty(state: State<'_, AppState>, id: String, data: String) {
    let mut processes = state.processes.lock().unwrap();
    if let Some(proc) = processes.get_mut(&id) {
        let _ = write!(proc.writer, "{}", data);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState::new(app.handle()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_task,
            get_tasks,
            delete_task,
            start_task,
            stop_task,
            resize_pty,
            write_to_pty,
            get_log_history,
            clear_log_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
