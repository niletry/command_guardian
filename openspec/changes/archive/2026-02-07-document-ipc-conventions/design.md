# Design: Document IPC and Development Conventions

## Context
The project has recently been rewritten from Python to Rust using Tauri. During implementation, several naming conflicts and race conditions were encountered due to implicit Tauri behavior. This design documents the standardized approach to bridge the frontend and backend.

## Goals / Non-Goals

**Goals:**
- Eliminate "Missing Key" errors in Tauri IPC.
- Prevent Mutex deadlocks in the backend.
- Standardize error handling for better developer experience.

**Non-Goals:**
- Changing the existing UI layout.
- Adding new functional features during this documentation phase.

## Decisions

### 1. Unified Naming Strategy
- **Decision**: Use JS `camelCase` for arguments and Rust `snake_case` for data structures.
- **Rationale**: Tauri's `invoke` automatically converts JS keys to Rust fields if correctly mapped. This preserves idiomatic styles in both languages.

### 2. Lock Management Pattern
- **Decision**: Use explicit block scopes for all Mutex guards.
- **Rationale**: Prevents accidental deadlocks when calling helper functions (like `save_config`) that also require locks.

### 3. Result-based Error Propagation
- **Decision**: All Tauri commands return `Result<T, String>`.
- **Rationale**: Maps directly to JS Promises, making `try-catch` on the frontend natural and effective.

## Risks / Trade-offs
- **Risk**: Developer forgets the camelCase rule for a new command.
- **Mitigation**: This spec will serve as the primary directive for AI Agent code generation.
