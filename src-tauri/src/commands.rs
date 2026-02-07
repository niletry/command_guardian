use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};

use crate::models::{TaskConfig, TaskStatus, TaskView};
use crate::state::{stop_task_internal, AppState, RunningProcess};

#[tauri::command]
pub fn create_task(
    state: State<'_, AppState>,
    name: String,
    command: String,
    tag: String,
    auto_retry: bool,
    env_vars: Option<HashMap<String, String>>,
) -> String {
    println!(">>> BACKEND: create_task name={}", name);
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

    {
        let mut tasks = state.tasks.lock().unwrap();
        let mut statuses = state.statuses.lock().unwrap();
        tasks.insert(id.clone(), config);
        statuses.insert(id.clone(), status);
    }

    state.save_config();
    id
}

#[tauri::command]
pub fn get_tasks(state: State<'_, AppState>) -> Vec<TaskView> {
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
pub fn delete_task(state: State<'_, AppState>, id: String) -> Result<(), String> {
    println!(">>> BACKEND: delete_task id={}", id);
    stop_task_internal(&state, &id);

    {
        let mut tasks = state.tasks.lock().unwrap();
        let mut statuses = state.statuses.lock().unwrap();
        tasks.remove(&id);
        statuses.remove(&id);
    }

    state.save_config();

    let log_path = state.log_dir.join(format!("{}.log", id));
    if log_path.exists() {
        let _ = std::fs::remove_file(log_path);
    }
    Ok(())
}

#[tauri::command]
pub fn start_task(state: State<'_, AppState>, app: AppHandle, id: String) -> Result<(), String> {
    println!(">>> BACKEND: start_task id={}", id);
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
                    if let Some(ref mut f) = log_file {
                        let _ = f.write_all(&buffer[..n]);
                        let _ = f.flush();
                    }
                    let data = String::from_utf8_lossy(&buffer[..n]).to_string();
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
pub fn stop_task(state: State<'_, AppState>, app: AppHandle, id: String) -> Result<(), String> {
    println!(">>> BACKEND: stop_task id={}", id);
    stop_task_internal(&state, &id);
    let _ = app.emit("task-updated", id);
    Ok(())
}

#[tauri::command]
pub fn update_task(
    state: State<'_, AppState>,
    id: String,
    name: String,
    command: String,
    tag: String,
    auto_retry: bool,
    env_vars: Option<HashMap<String, String>>,
) -> Result<(), String> {
    println!(">>> BACKEND: update_task id={}", id);
    {
        let mut tasks = state.tasks.lock().unwrap();
        if let Some(config) = tasks.get_mut(&id) {
            config.name = name;
            config.command = command;
            config.tag = tag;
            config.auto_retry = auto_retry;
            config.env_vars = env_vars;
        } else {
            return Err("Task not found".to_string());
        }
    }

    state.save_config();
    Ok(())
}

#[tauri::command]
pub fn resize_pty(state: State<'_, AppState>, id: String, rows: u16, cols: u16) {
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

use std::io::SeekFrom;

#[tauri::command]
pub fn get_log_history(state: State<'_, AppState>, id: String) -> String {
    let log_path = state.log_dir.join(format!("{}.log", id));
    if !log_path.exists() {
        return String::new();
    }

    let mut file = match std::fs::File::open(&log_path) {
        Ok(f) => f,
        Err(_) => return String::new(),
    };

    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    let read_size = std::cmp::min(size, 50_000);

    if let Err(_) = file.seek(SeekFrom::End(-(read_size as i64))) {
        let _ = file.seek(SeekFrom::Start(0));
    }

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap_or_default();
    String::from_utf8_lossy(&buffer).to_string()
}

#[tauri::command]
pub fn clear_log_history(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let log_path = state.log_dir.join(format!("{}.log", id));
    if log_path.exists() {
        let _ = std::fs::remove_file(log_path);
    }
    Ok(())
}

#[tauri::command]
pub fn write_to_pty(state: State<'_, AppState>, id: String, data: String) {
    let mut processes = state.processes.lock().unwrap();
    if let Some(proc) = processes.get_mut(&id) {
        let _ = write!(proc.writer, "{}", data);
    }
}
