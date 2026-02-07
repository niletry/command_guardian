## ADDED Requirements

### Requirement: Complete Task Deletion
The system SHALL ensure that deleting a task performs a complete cleanup of all associated resources.

#### Scenario: Deleting a task with running process
- **WHEN** `delete_task` is called for a task that is currently running
- **THEN** the system MUST terminate the running process
- **AND** the system MUST remove the task from the internal state
- **AND** the system MUST delete the task's log file from disk
- **AND** the system MUST save the updated configuration
