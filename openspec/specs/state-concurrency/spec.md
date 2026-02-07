# state-concurrency Specification

## Purpose
TBD - created by archiving change document-ipc-conventions. Update Purpose after archive.
## Requirements
### Requirement: Mutex Locking Order
The backend SHALL never hold a lock on `AppState::tasks` or `AppState::processes` while performing blocking disk IO or calling `save_config()`.

#### Scenario: Updating a task and saving
- **WHEN** `update_task` is called
- **THEN** it MUST release the Mutex lock on `tasks` before calling `state.save_config()`.

### Requirement: Atomic State Transitions
The backend SHALL update both `tasks` and `statuses` maps consistently during creation or deletion.

#### Scenario: Deleting a task
- **WHEN** `delete_task` is called
- **THEN** it MUST remove the entry from both `tasks` and `statuses` within the same logical operation.

