## 1. 后端修复 (Rust)

- [x] 1.1 修改 `delete_task` 返回 `Result<(), String>` 并增强错误处理。
- [x] 1.2 在 `delete_task` 成功后，通过 `AppHandle` 发送 `task-updated` 事件。
- [x] 1.3 修复 `create_task` 和 `update_task` 的参数命名，使其能够正确接收前端的 camelCase 参数。

## 2. 前端修复 (TypeScript)

- [x] 2.1 更新 `App.tsx` 中的 `handleDelete`，显式处理删除失败的情况并提供更好的错误提示。
- [x] 2.2 确保 `handleCreateOrUpdate` 发送的参数与后端修复后的接口一致。

## 3. 验证

- [x] 3.1 运行应用并测试删除功能，验证任务列表是否实时刷新、配置是否保存、日志文件是否删除。
- [x] 3.2 验证创建和更新任务时，环境变量和自动重试等参数是否正确保存。

