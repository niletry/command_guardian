## Context

Currently, `TerminalView.tsx` uses `listen('task-output', (event: any) => ...)` which provides no type information for the event payload.

## Goals / Non-Goals

**Goals:**
- Define a reusable interface for the terminal output payload.
- Update the component to use this interface in the event listener.

**Non-Goals:**
- Refactoring the entire terminal implementation.
- Changing the backend emission logic.

## Decisions

### Decision 1: Placement of the Interface

I'll define the `TaskOutputPayload` interface directly in `TerminalView.tsx` to keep it close to its primary consumer. Given the current structure, local definition is simplest for this scope.
