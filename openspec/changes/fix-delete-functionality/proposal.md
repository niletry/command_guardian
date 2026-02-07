## Why

用户报告“删除任务”功能目前无法正常工作。这会导致无法清理不需要的任务配置和日志文件，影响用户体验并可能导致界面数据冗余。

## What Changes

- **修复**：调查并修复 `src-tauri/src/lib.rs` 中的 `delete_task` 命令，确保其能正确停止进程、移除状态并删除磁盘上的日志文件。
- **修复**：检查 `src/App.tsx` 中的调用逻辑，确保前端能正确处理删除后的状态更新。
- **验证**：确保删除操作后，任务列表能实时刷新且相关资源已被释放。

## Capabilities

### New Capabilities
- `task-deletion`: Formalizes the requirements for deleting a task, including process termination and resource cleanup.

### Modified Capabilities
- `state-concurrency`: 验证在多任务并行情况下，删除单个任务不会影响其他任务的状态。

## Impact

- `src-tauri/src/lib.rs`: `delete_task` 命令实现。
- `src/App.tsx`: `handleDelete` 函数。
- `openspec/specs/state-concurrency/spec.md`: 相关的状态一致性验证。
