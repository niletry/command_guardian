# Proposal: Document IPC and Development Conventions

## Why
As the project migrates from Python to Rust (Tauri), several implicit conventions regarding communication (camelCase vs snake_case), concurrency (locking order), and error handling have emerged. Documenting these ensures future development remains consistent and error-free.

## What Changes
- Add formal specification for Tauri IPC communication rules.
- Add formal specification for backend state management and concurrency rules.
- Add formal specification for frontend-backend error handling patterns.

## Capabilities

### New Capabilities
- `ipc-protocol`: Rules for data serialization and command arguments between JS and Rust.
- `state-concurrency`: Guidelines for Mutex locking and state persistence.
- `error-handling`: Standard patterns for Result types and UI error notifications.

### Modified Capabilities
- None

## Impact
- `src-tauri/src/lib.rs`: Implementation must align with the spec.
- `src/App.tsx`: Implementation must align with the spec.
- All future development by AI Agents or humans.
