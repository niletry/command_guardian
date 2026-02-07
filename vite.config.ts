import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  // 防止 vite 占用随机端口，固定为 tauri 预期的 1420
  server: {
    port: 1420,
    strictPort: true,
    host: true,
  },
})
