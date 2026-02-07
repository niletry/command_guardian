# ipc-protocol Specification

## Purpose
TBD - created by archiving change document-ipc-conventions. Update Purpose after archive.
## Requirements
### Requirement: CamelCase for Command Arguments
The frontend MUST use `camelCase` for keys in the payload object when invoking Tauri commands.

#### Scenario: Task creation with camelCase
- **WHEN** the frontend calls `invoke("create_task", { autoRetry: true, envVars: {} })`
- **THEN** the backend correctly maps these to `auto_retry` and `env_vars` Rust parameters.

### Requirement: Snake_case for Data Models
The backend SHALL serialize Rust structs using `snake_case`, and the frontend MUST consume them as such.

#### Scenario: Task list retrieval
- **WHEN** the backend returns a `TaskConfig` struct with `auto_retry` field
- **THEN** the frontend React state stores and accesses it as `task.config.auto_retry`.

