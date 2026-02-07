# Command Guardian v2

高性能通用命令托管与进程守护桌面应用。

基于 **Tauri v2 + Rust + React** 打造，彻底告别臃肿，拥抱原生性能。

## 核心特性

- **极致轻量**：安装包仅约 8MB，内存占用极低。
- **原生 Rust 后端**：使用 Rust `portable-pty` 实现高性能进程管理。
- **现代极客 UI**：基于 React 和 Tailwind CSS 打造的深色模式 Dashboard。
- **专业终端回显**：集成 `xterm.js`，支持 ANSI 颜色、实时交互和自动缩放。
- **标签化管理**：支持按分类过滤任务，界面清新直观。

## 开发环境

- **Rust**: 1.77+
- **Node.js**: 18+
- **Cargo**: Rust 包管理器
- **npm**: Node 包管理器

## 运行与开发

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

## 项目结构

```
.
├── src/                # React 前端源代码
├── src-tauri/          # Rust 后端核心逻辑
├── public/             # 静态资源
├── package.json        # 前端依赖与脚本
└── README.md           # 项目文档
```

## 技术栈

- **Frontend**: React, TypeScript, Tailwind CSS, Lucide React, xterm.js
- **Backend**: Rust, Tauri v2, portable-pty, tokio
- **Build Tool**: Vite
