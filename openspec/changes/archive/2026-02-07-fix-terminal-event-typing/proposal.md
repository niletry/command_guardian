## Why

The `task-output` event listener in `TerminalView.tsx` currently uses the `any` type for its payload. This bypasses TypeScript's safety net and makes the code harder to maintain, as the relationship between the Rust backend's emitted data and the frontend's consumption is implicit rather than formal.

## What Changes

- The system SHALL define a `TaskOutputPayload` interface that matches the structure emitted by the Rust backend.
- The system SHALL replace the `any` type in `TerminalView.tsx` with this new interface.

## Capabilities

### New Capabilities
- `terminal-output-typing`: Formalizes the contract between the backend and the terminal view via a shared type definition.

### Modified Capabilities

## Impact

- `src/components/TerminalView.tsx`: The event listener will be updated to use the new interface.
