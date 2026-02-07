# Command Guardian v2

作为开发者，你是否经常遇到这些场景？
- 为了运行不同的服务（API, Frontend, Watchers），终端里塞满了七八个标签页，找都找不着。
- 某个开发辅助命令需要反复开启/关闭，每次都要在历史记录里翻找那串带长参数的命令。
- 下班关机前想快速关掉所有后台进程，或者第二天上班想一键恢复开发环境。

**Command Guardian** 就是为了解决这些“琐碎而高频”的操作而生的。它将枯燥的命令行变成了直观的卡片工作台，让你通过简单的点击就能完成命令的托管与进程管理。

![主界面](./screenshots/main.png)

## 为什么你需要它？

### 🚀 告别“命令记忆”焦虑
不再需要记住复杂的命令。将带环境变量、长参数的 Shell 脚本一次性配置好，以后只需点击 **"Start"**。

### 📊 统一的可视化看板
在一个界面看到所有服务的运行状态：谁在运行？运行了多久？PID 是多少？再也不用反复执行 `ps aux | grep` 来确认进程状态。

### 🖥️ 沉浸式的终端体验
点击卡片上的终端图标，即可弹出实时输出窗口。内置专业的 `xterm.js` 支持 ANSI 颜色显示和实时输入交互，体验和原生终端一样丝滑。

### 🏷️ 灵活的任务分类
你可以按标签（Tag）对任务进行分类。比如点击 "Prod" 查看生产环境任务，或点击 "Dev" 快速启动所有本地开发相关的命令。

---

## 使用指南

### 1. 添加任务
点击左侧边栏底部的 **"New Task"** 按钮。在弹出的窗口中填写：
- **Name**: 任务显示的名称（如: 前端开发服务）。
- **Command**: 要执行的 Shell 命令（如 `npm run dev`）。
- **Tag**: 分类标签（如: Frontend, API）。
- **Environment Variables**: 支持配置自定义环境变量，格式为 `KEY=VALUE`，每行一个。

![添加任务](./screenshots/newtask.png)

### 2. 管理进程
- **Start/Stop**: 点击卡片底部的开关或按钮，快速启停进程。
- **Uptime**: 实时观察进程运行时间，确认其稳定性。
- **Terminal**: 进入交互式终端查看日志输出，支持直接输入指令。

### 3. 日志管理
- 所有的输出都会自动持久化到系统应用目录下的 `logs/` 文件夹。
- 觉得日志太乱？点击窗口中的 **"Clear Log"** 即可一键物理删除日志文件。

---

## 开发与构建

### 环境要求
- **Rust**: 1.77+
- **Node.js**: 18+

### 快速开始
1. **安装依赖**：
   ```bash
   npm install
   ```
2. **启动开发模式**：
   ```bash
   npm run tauri dev
   ```
3. **构建安装包 (DMG)**：
   ```bash
   npm run tauri build
   ```

## 技术栈
- **Frontend**: React 19, TypeScript, Tailwind CSS v4, Lucide Icons, xterm.js
- **Backend**: Rust, Tauri v2, portable-pty, Tokio
- **Build Tool**: Vite, Cargo

---
*Command Guardian - 让命令托管更简单、更原生。*
