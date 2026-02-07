## MODIFIED Requirements

### Requirement: Atomic State Transitions
The backend SHALL update both `tasks` and `statuses` maps consistently during creation or deletion, ensuring that operations on one task SHALL NOT affect the state of other tasks.

#### Scenario: Deleting a task
- **WHEN** `delete_task` is called
- **THEN** it MUST remove the entry from both `tasks` and `statuses` within the same logical operation.
- **AND** the state of other tasks MUST remain unchanged.
