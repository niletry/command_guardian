# Development Documentation: Communication & Conventions

This document outlines the architecture, communication protocols, and naming conventions used in **Command Guardian v2** to ensure consistency between the Rust backend and React frontend.

## 1. Architecture Overview

Command Guardian follows a decoupled architecture using **Tauri v2**:
- **Backend (Rust)**: Manages process lifecycles (via `portable-pty`), file-based logging, and configuration persistence (`config.json`).
- **Frontend (React)**: Provides a modern Dashboard UI, handles user interaction, and renders real-time logs via `xterm.js`.
- **Bridge**: Communication happens via **Tauri IPC (Inter-Process Communication)** using `invoke` (Frontend to Backend) and `emit/listen` (Backend to Frontend).

## 2. Naming Conventions (Critical)

To avoid "Missing Key" errors, follow these strict rules due to Tauri's automatic serialization:

### Data Structures (JSON/Models)
- **Rust Side**: Uses `snake_case` (e.g., `auto_retry`).
- **Frontend Side**: Receives and processes `snake_case`.
- **Reason**: When Rust structs are serialized to JSON and sent to the UI, they maintain their original field names.

### Command Arguments (IPC)
- **Rust Side**: Defined as `snake_case` (e.g., `fn create_task(auto_retry: bool)`).
- **Frontend Side**: **MUST** use `camelCase` when calling `invoke`.
- **Example**: `invoke("create_task", { autoRetry: true })`.
- **Reason**: Tauri automatically maps JS `camelCase` keys to Rust `snake_case` parameters.

## 3. API Reference (Tauri Commands)

| Command | Arguments (JS Side) | Return Type | Description |
| :--- | :--- | :--- | :--- |
| `get_tasks` | None | `TaskView[]` | Fetches all tasks and their current statuses. |
| `create_task` | `{ name, command, tag, autoRetry, envVars }` | `string` (ID) | Creates a new task and persists to config. |
| `update_task` | `{ id, name, command, tag, autoRetry, envVars }` | `Result<void, string>` | Updates an existing task configuration. |
| `delete_task` | `{ id }` | `void` | Stops process, removes task, and deletes log file. |
| `start_task` | `{ id }` | `Result<void, string>` | Spawns a PTY process for the given task. |
| `stop_task` | `{ id }` | `void` | Kills the process and updates status. |
| `get_log_history`| `{ id }` | `string` | Reads the last 50KB of the task's log file. |
| `clear_log_history`| `{ id }` | `void` | Deletes the physical log file from disk. |
| `write_to_pty` | `{ id, data }` | `void` | Sends string input to the running PTY process. |
| `resize_pty` | `{ id, rows, cols }` | `void` | Resizes the PTY terminal dimensions. |

## 4. Real-time Events (Backend -> Frontend)

The backend emits events to keep the UI in sync without polling:

- **`task-updated`**: Emitted when a task starts, stops, or crashes. Payload: `task_id`.
- **`task-output`**: Emitted when new data is read from the PTY. Payload: `{ id: string, data: string }`.

## 5. Persistence & Storage

- **Config**: `~/Library/Application Support/com.commandguardian.app/config.json`
- **Logs**: `~/Library/Application Support/com.commandguardian.app/logs/[task_id].log`

## 6. Common Pitfalls

1. **Deadlocks**: Never call `state.save_config()` while holding a lock on `state.tasks`. Always wrap the modification in a scope `{ ... }` so the lock is released before saving.
2. **Missing required key**: If you see this error, check if you are using `snake_case` instead of `camelCase` in the `invoke` argument object.
3. **Terminal ANSI**: When writing to `xterm.js`, ensure `convertEol: true` is set in terminal options to correctly handle PTY line endings (`\r\n`).
