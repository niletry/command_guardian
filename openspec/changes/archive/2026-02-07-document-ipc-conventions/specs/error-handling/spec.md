## ADDED Requirements

### Requirement: Standardized Result Return
The backend MUST return `Result<(), String>` for operations that can fail (start, stop, delete, update).

#### Scenario: Failed process start
- **WHEN** a task fails to start due to invalid command
- **THEN** the backend returns an `Err(message)` which is caught by the frontend.

### Requirement: Frontend Error Visibility
The frontend MUST wrap all `invoke` calls in `try-catch` blocks and display errors to the user.

#### Scenario: Displaying backend error
- **WHEN** the backend returns an error message
- **THEN** the frontend displays an `alert()` or UI notification with the message.
