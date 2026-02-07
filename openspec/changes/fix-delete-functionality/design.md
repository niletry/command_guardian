## Context

用户报告删除功能无法正常工作。经过调查，虽然 `src-tauri/src/lib.rs` 中的 `delete_task` 逻辑看起来是正确的（停止进程、移除状态、保存配置、删除日志），但可能存在以下问题：
1. **反馈缺失**：`delete_task` 没有返回 `Result`，如果发生内部错误（如 Mutex 毒化），前端可能无法感知。
2. **IPC 命名不一致**：在 `create_task` 和 `update_task` 中发现了前端发送 camelCase 而后端期望 snake_case 的问题。虽然 `delete_task` 的 `id` 参数不受影响，但这反映了 IPC 层的不一致性。
3. **竞态条件**：删除操作与配置保存、文件删除之间可能存在细微的异步冲突。

## Goals / Non-Goals

**Goals:**
- 修复 `delete_task` 及其相关的 IPC 逻辑。
- 确保删除操作后，前端能得到明确的成功/失败反馈。
- 统一 IPC 参数命名规范（前端统一发送 snake_case 以匹配 Rust 参数，或后端增加重命名注解）。

**Non-Goals:**
- 大规模重构状态管理系统。
- 改变现有的文件存储格式。

## Decisions

### Decision 1: 增强后端指令的健壮性
将 `delete_task` 修改为返回 `Result<(), String>`。这样如果删除过程中发生任何错误，前端都能通过 `catch` 捕获并显示具体的错误信息。

### Decision 2: 修复参数命名不一致
由于前端 `App.tsx` 已经在使用 `autoRetry` 和 `envVars` 等 camelCase 命名，且 `delete_task` 使用的是 `id`，我将对 Rust 中的 `create_task` 和 `update_task` 参数进行修复，并确保 `delete_task` 的调用始终传递正确的参数名。

### Decision 3: 增加删除后的显式刷新
虽然前端已经调用了 `refreshTasks`，但在后端完成 `delete_task` 后显式发出一个 `task-updated` 事件（携带已删除的 ID），可以帮助前端其他潜在的监听者清理缓存。

## Risks / Trade-offs

- [Risk] → 删除正在运行的任务时，日志文件可能被占用导致删除失败。
- [Mitigation] → 在 `delete_task` 中先执行 `stop_task_internal`，并在删除文件前进行简短重试或捕获错误。
